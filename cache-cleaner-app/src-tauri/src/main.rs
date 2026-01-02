#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod utils;
mod payment;
mod monetization;

use cache::{
    CacheInfo, CacheType, CleanResult, IndexedDbCleanResult, IndexedDbItem,
    LargeCacheEntry, LargeCachesCleanResult, NpmCacheEntry, NpmCachesCleanResult,
    config::AppConfig,
    custom_scanner::CustomScannerConfig,
    registry::ScannerRegistry,
    scanner_trait::{ScanResult, CleanResultGeneric},
    smart_suggestions::{FolderSuggestion, SmartSuggestionsCleanResult},
};
use monetization::{AdError, AdManager, PremiumService, PaymentSession};
use payment::{PaddleClient, PurchaseReceipt};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::OnceCell;

static REGISTRY: OnceCell<Arc<ScannerRegistry>> = OnceCell::const_new();
static PREMIUM_SERVICE: OnceLock<Arc<Mutex<Option<Arc<PremiumService>>>>> = OnceLock::new();
static AD_MANAGER: OnceLock<Arc<Mutex<Option<Arc<AdManager>>>>> = OnceLock::new();

async fn get_registry() -> &'static Arc<ScannerRegistry> {
    REGISTRY.get_or_init(|| async {
        let registry = Arc::new(ScannerRegistry::new());
        // Load saved custom scanners
        if let Ok(config) = AppConfig::load() {
            for scanner_config in config.custom_scanners {
                let _ = registry.register(scanner_config).await;
            }
        }
        registry
    }).await
}

/// Initialize PremiumService (called once)
async fn init_premium_service() -> Result<Arc<PremiumService>, String> {
    eprintln!("[MonetizationMiddleware] Initializing PremiumService...");
    
    // Initialize storage (blocking I/O wrapped in spawn_blocking for Send safety)
    let storage = tokio::task::spawn_blocking(|| {
        monetization::PremiumStorage::with_default_path()
            .map_err(|e| format!("Failed to create PremiumStorage: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    let storage = Arc::new(storage);
    
    // Initialize Paddle client (blocking I/O wrapped in spawn_blocking for Send safety)
    let paddle_client = tokio::task::spawn_blocking(|| {
        PaddleClient::from_config()
            .map_err(|e| format!("Failed to create PaddleClient: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    let paddle_client = Arc::new(paddle_client);
    
    // Initialize payment manager
    let payment_manager = Arc::new(
        monetization::PaymentManager::new(paddle_client, storage.clone())
    );
    
    // Initialize premium service
    let service = Arc::new(
        PremiumService::new(storage, payment_manager, None)
            .map_err(|e| format!("Failed to create PremiumService: {}", e))?
    );
    
    // Initialize the service (restore purchases, etc.)
    if let Err(e) = service.initialize().await {
        eprintln!("[MonetizationMiddleware] Warning: PremiumService initialization failed: {:?}", e);
    }
    
    eprintln!("[MonetizationMiddleware] PremiumService initialized successfully");
    Ok(service)
}

/// Get or initialize the PremiumService singleton
async fn get_premium_service() -> Result<Arc<PremiumService>, String> {
    let lock = PREMIUM_SERVICE.get_or_init(|| {
        Arc::new(Mutex::new(None))
    });
    
    // Fast path: check if already initialized
    {
        let service = lock.lock().unwrap();
        if let Some(ref s) = *service {
            return Ok(s.clone());
        }
    }
    
    // Slow path: initialize (drop lock before await)
    let new_service = init_premium_service().await?;
    
    // Update with lock
    {
        let mut service = lock.lock().unwrap();
        // Double-check after acquiring lock
        if let Some(ref s) = *service {
            return Ok(s.clone());
        }
        *service = Some(new_service.clone());
    }
    Ok(new_service)
}

/// Get or initialize the AdManager singleton
async fn get_ad_manager() -> Result<Arc<AdManager>, String> {
    let lock = AD_MANAGER.get_or_init(|| {
        Arc::new(Mutex::new(None))
    });
    
    // Fast path: check if already initialized
    {
        let manager = lock.lock().unwrap();
        if let Some(ref m) = *manager {
            return Ok(m.clone());
        }
    }
    
    // Slow path: initialize (drop lock before await)
    // Ensure premium service is initialized first
    let premium_service = get_premium_service().await?;
    let new_manager = Arc::new(AdManager::with_default_config(premium_service));
    
    // Update with lock
    {
        let mut manager = lock.lock().unwrap();
        // Double-check after acquiring lock
        if let Some(ref m) = *manager {
            return Ok(m.clone());
        }
        *manager = Some(new_manager.clone());
    }
    Ok(new_manager)
}

/// Get or initialize a PaymentManager instance
/// 
/// Creates a PaymentManager with shared storage and Paddle client.
/// This is used by payment-related commands.
async fn get_payment_manager() -> Result<Arc<monetization::PaymentManager>, String> {
    // Initialize storage (blocking I/O wrapped in spawn_blocking for Send safety)
    let storage = tokio::task::spawn_blocking(|| {
        monetization::PremiumStorage::with_default_path()
            .map_err(|e| format!("Failed to create PremiumStorage: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    let storage = Arc::new(storage);
    
    // Initialize Paddle client (blocking I/O wrapped in spawn_blocking for Send safety)
    let paddle_client = tokio::task::spawn_blocking(|| {
        PaddleClient::from_config()
            .map_err(|e| format!("Failed to create PaddleClient: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;
    let paddle_client = Arc::new(paddle_client);
    
    // Initialize payment manager
    Ok(Arc::new(
        monetization::PaymentManager::new(paddle_client, storage)
    ))
}

#[derive(Serialize, Deserialize)]
pub struct PermissionStatus {
    pub full_disk_access: bool,
    pub home_accessible: bool,
}

#[tauri::command]
async fn scan_caches() -> Result<Vec<CacheInfo>, String> {
    cache::scanner::scan_all().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_cache_size(cache_type: String) -> Result<u64, String> {
    let ct = CacheType::from_str(&cache_type).map_err(|e| e.to_string())?;
    cache::scanner::get_size(&ct).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn clean_cache(cache_type: String, dry_run: bool) -> Result<CleanResult, String> {
    println!(
        "[Rust] clean_cache called: cache_type={}, dry_run={}",
        cache_type, dry_run
    );

    let ct = CacheType::from_str(&cache_type).map_err(|e| {
        eprintln!("[Rust] Failed to parse CacheType from '{}': {}", cache_type, e);
        e.to_string()
    })?;

    let result = cache::cleaner::clean(&ct, dry_run)
        .await
        .map_err(|e| {
            eprintln!(
                "[Rust] cache::cleaner::clean error for {:?}, dry_run={}: {}",
                ct, dry_run, e
            );
            e.to_string()
        })?;

    println!(
        "[Rust] clean_cache finished: type={:?}, freed_bytes={}, items_removed={}, dry_run={}",
        result.cache_type, result.freed_bytes, result.items_removed, result.dry_run
    );

    Ok(result)
}

/// Check if monetization is enabled via feature flag
/// 
/// Checks environment variable ENABLE_MONETIZATION (default: true)
fn is_monetization_enabled() -> bool {
    std::env::var("ENABLE_MONETIZATION")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true)
}

/// Clean cache with monetization middleware
/// 
/// This command intercepts deletion requests and enforces ad/payment requirements:
/// - Checks premium status before deletion
/// - If not premium: signals frontend to show ad and waits for completion
/// - If premium: proceeds immediately
/// - Integrates with existing cache cleaner
/// 
/// Feature flag: Set ENABLE_MONETIZATION=false to disable monetization and fall back to regular cleaning
#[tauri::command]
async fn clean_cache_with_monetization(
    cache_type: String,
    dry_run: bool,
) -> Result<CleanResult, String> {
    eprintln!(
        "[MonetizationMiddleware] clean_cache_with_monetization called: cache_type={}, dry_run={}",
        cache_type, dry_run
    );

    // Check feature flag
    if !is_monetization_enabled() {
        eprintln!("[MonetizationMiddleware] Monetization disabled via feature flag, falling back to regular clean_cache");
        return clean_cache(cache_type, dry_run).await;
    }

    // Parse cache type
    let ct = CacheType::from_str(&cache_type).map_err(|e| {
        eprintln!(
            "[MonetizationMiddleware] Failed to parse CacheType from '{}': {}",
            cache_type, e
        );
        e.to_string()
    })?;

    // Get premium service
    let premium_service = get_premium_service().await.map_err(|e| {
        eprintln!("[MonetizationMiddleware] Failed to get PremiumService: {}", e);
        format!("Monetization service unavailable: {}", e)
    })?;

    // Check premium status
    let is_premium = premium_service.is_premium().await;
    eprintln!(
        "[MonetizationMiddleware] Premium status checked: is_premium={}, cache_type={:?}",
        is_premium, ct
    );

    if !is_premium {
        // User is not premium - require ad before deletion
        eprintln!("[MonetizationMiddleware] User is not premium, requesting ad...");
        
        // Get ad manager
        let ad_manager = get_ad_manager().await.map_err(|e| {
            eprintln!("[MonetizationMiddleware] Failed to get AdManager: {}", e);
            format!("Ad service unavailable: {}", e)
        })?;

        // Request ad
        match ad_manager.request_ad().await {
            Ok(ad_request) => {
                eprintln!(
                    "[MonetizationMiddleware] Ad requested: ad_id={}, provider={}, format={}",
                    ad_request.ad_id, ad_request.provider, ad_request.format
                );

                // Log monetization event: ad required
                eprintln!(
                    "[MonetizationMiddleware] Monetization event: ad_required, ad_id={}, cache_type={:?}",
                    ad_request.ad_id, ct
                );

                // Wait for ad completion
                eprintln!(
                    "[MonetizationMiddleware] Waiting for ad completion: ad_id={}",
                    ad_request.ad_id
                );
                
                match ad_manager.wait_for_ad_completion(&ad_request.ad_id).await {
                    Ok(()) => {
                        eprintln!(
                            "[MonetizationMiddleware] Ad completed successfully: ad_id={}",
                            ad_request.ad_id
                        );
                        // Log monetization event: ad completed
                        eprintln!(
                            "[MonetizationMiddleware] Monetization event: ad_completed, ad_id={}, cache_type={:?}",
                            ad_request.ad_id, ct
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "[MonetizationMiddleware] Ad completion failed: ad_id={}, error={:?}",
                            ad_request.ad_id, e
                        );
                        // Log monetization event: ad failed
                        eprintln!(
                            "[MonetizationMiddleware] Monetization event: ad_failed, ad_id={}, error={:?}, cache_type={:?}",
                            ad_request.ad_id, e, ct
                        );
                        return Err(format!("Ad completion required but failed: {:?}", e));
                    }
                }
            }
            Err(AdError::NotRequired) => {
                // User became premium between checks, or ad not required
                eprintln!(
                    "[MonetizationMiddleware] Ad not required (user may have become premium)"
                );
            }
            Err(e) => {
                eprintln!(
                    "[MonetizationMiddleware] Failed to request ad: error={:?}",
                    e
                );
                // Log monetization event: ad request failed
                eprintln!(
                    "[MonetizationMiddleware] Monetization event: ad_request_failed, error={:?}, cache_type={:?}",
                    e, ct
                );
                return Err(format!("Failed to request ad: {:?}", e));
            }
        }
    } else {
        // User is premium - proceed immediately
        eprintln!(
            "[MonetizationMiddleware] User is premium, proceeding with deletion immediately"
        );
        // Log monetization event: premium user
        eprintln!(
            "[MonetizationMiddleware] Monetization event: premium_user_skip_ad, cache_type={:?}",
            ct
        );
    }

    // Proceed with cache cleaning
    eprintln!(
        "[MonetizationMiddleware] Proceeding with cache cleaning: cache_type={:?}, dry_run={}",
        ct, dry_run
    );

    let result = cache::cleaner::clean(&ct, dry_run)
        .await
        .map_err(|e| {
            eprintln!(
                "[MonetizationMiddleware] cache::cleaner::clean error for {:?}, dry_run={}: {}",
                ct, dry_run, e
            );
            e.to_string()
        })?;

    eprintln!(
        "[MonetizationMiddleware] clean_cache_with_monetization finished: type={:?}, freed_bytes={}, items_removed={}, dry_run={}, is_premium={}",
        result.cache_type, result.freed_bytes, result.items_removed, result.dry_run, is_premium
    );

    // Log monetization event: deletion completed
    eprintln!(
        "[MonetizationMiddleware] Monetization event: deletion_completed, cache_type={:?}, freed_bytes={}, items_removed={}, is_premium={}",
        ct, result.freed_bytes, result.items_removed, is_premium
    );

    Ok(result)
}

#[tauri::command]
async fn check_chrome_running() -> Result<bool, String> {
    utils::permissions::is_chrome_running().map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_permissions() -> Result<PermissionStatus, String> {
    Ok(PermissionStatus {
        full_disk_access: utils::permissions::has_full_disk_access(),
        home_accessible: utils::permissions::can_access_home(),
    })
}



#[tauri::command]
async fn scan_indexed_db_items(
    threshold_mb: Option<u64>,
) -> Result<Vec<IndexedDbItem>, String> {
    // Default alert threshold is 10MB if not specified.
    let mb = threshold_mb.unwrap_or(10);
    let threshold_bytes = mb * 1024 * 1024;
    cache::indexeddb::scan_indexed_db(threshold_bytes).map_err(|e| e.to_string())
}

#[tauri::command]
async fn clean_indexed_db_items(
    paths: Vec<String>,
    dry_run: bool,
) -> Result<IndexedDbCleanResult, String> {
    cache::indexeddb::clean_indexed_db_items(paths, dry_run).map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_large_caches() -> Result<Vec<LargeCacheEntry>, String> {
    cache::large_caches::scan_large_caches()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_large_caches(paths: Vec<String>) -> Result<LargeCachesCleanResult, String> {
    cache::large_caches::remove_large_caches(paths)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_npm_caches() -> Result<Vec<NpmCacheEntry>, String> {
    cache::npm_caches::scan_npm_caches()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_npm_caches(paths: Vec<String>) -> Result<NpmCachesCleanResult, String> {
    cache::npm_caches::remove_npm_caches(paths)
        .await
        .map_err(|e| e.to_string())
}

// === Custom Scanner Commands ===

#[tauri::command]
async fn register_custom_scanner(config: CustomScannerConfig) -> Result<(), String> {
    let registry = get_registry().await;
    registry.register(config.clone()).await.map_err(|e| e.to_string())?;
    
    // Persist to config
    let mut app_config = AppConfig::load().unwrap_or_default();
    app_config.add_scanner(config);
    app_config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn list_custom_scanners() -> Result<Vec<CustomScannerConfig>, String> {
    let registry = get_registry().await;
    Ok(registry.list().await)
}

#[tauri::command]
async fn remove_custom_scanner(id: String) -> Result<bool, String> {
    let registry = get_registry().await;
    let removed = registry.unregister(&id).await;
    
    if removed {
        let mut app_config = AppConfig::load().unwrap_or_default();
        app_config.remove_scanner(&id);
        app_config.save().map_err(|e| e.to_string())?;
    }
    Ok(removed)
}

#[tauri::command]
async fn scan_custom_caches() -> Result<Vec<ScanResult>, String> {
    let registry = get_registry().await;
    Ok(registry.scan_all_custom().await)
}

#[tauri::command]
async fn clean_custom_cache(id: String, dry_run: bool) -> Result<CleanResultGeneric, String> {
    let registry = get_registry().await;
    registry.clean_custom(&id, dry_run).await.map_err(|e| e.to_string())
}

// === Smart Suggestions Commands ===

#[tauri::command]
async fn scan_smart_suggestions(min_size_mb: Option<u64>, max_age_days: Option<u64>) -> Result<Vec<FolderSuggestion>, String> {
    cache::smart_suggestions::scan_suggestions(
        min_size_mb.unwrap_or(100),
        max_age_days.unwrap_or(30),
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_folder_suggestion_info(path: String) -> Result<FolderSuggestion, String> {
    cache::smart_suggestions::get_folder_info(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_smart_suggestions(paths: Vec<String>) -> Result<SmartSuggestionsCleanResult, String> {
    cache::smart_suggestions::remove_suggested_folders(paths).await.map_err(|e| e.to_string())
}

// === Monetization Commands ===

/// Check if the current user has premium status
/// 
/// Returns true if user is premium, false otherwise.
/// Premium users can skip ads and have unlimited cache deletions.
#[tauri::command]
async fn check_premium_status() -> Result<bool, String> {
    eprintln!("[Monetization] check_premium_status called");
    
    let premium_service = get_premium_service().await.map_err(|e| {
        eprintln!("[Monetization] Failed to get PremiumService: {}", e);
        format!("Monetization service unavailable: {}", e)
    })?;
    
    let is_premium = premium_service.is_premium().await;
    eprintln!("[Monetization] Premium status: {}", is_premium);
    
    Ok(is_premium)
}

/// Request an ad for display
/// 
/// Returns ad request information including ad_id, provider, and configuration.
/// If user is premium, returns an error indicating ad is not required.
#[tauri::command]
async fn request_ad() -> Result<monetization::AdRequest, String> {
    eprintln!("[Monetization] request_ad called");
    
    let ad_manager = get_ad_manager().await.map_err(|e| {
        eprintln!("[Monetization] Failed to get AdManager: {}", e);
        format!("Ad service unavailable: {}", e)
    })?;
    
    match ad_manager.request_ad().await {
        Ok(ad_request) => {
            eprintln!("[Monetization] Ad requested: ad_id={}", ad_request.ad_id);
            Ok(ad_request)
        }
        Err(AdError::NotRequired) => {
            eprintln!("[Monetization] Ad not required (user is premium)");
            Err("Ad not required: user is premium".to_string())
        }
        Err(e) => {
            eprintln!("[Monetization] Failed to request ad: {:?}", e);
            Err(format!("Failed to request ad: {:?}", e))
        }
    }
}

/// Notify backend that an ad has been completed
/// 
/// This should be called by the frontend after an ad finishes playing.
/// The backend will mark the ad as completed and allow cache deletion to proceed.
/// 
/// # Arguments
/// * `ad_id` - The ad ID returned from `request_ad()`
#[tauri::command]
async fn ad_completed(ad_id: String) -> Result<(), String> {
    eprintln!("[Monetization] ad_completed called: ad_id={}", ad_id);
    
    let ad_manager = get_ad_manager().await.map_err(|e| {
        eprintln!("[Monetization] Failed to get AdManager: {}", e);
        format!("Ad service unavailable: {}", e)
    })?;
    
    ad_manager.mark_ad_completed(&ad_id).await.map_err(|e| {
        eprintln!("[Monetization] Failed to mark ad as completed: ad_id={}, error={:?}", ad_id, e);
        format!("Failed to mark ad as completed: {:?}", e)
    })?;
    
    eprintln!("[Monetization] Ad marked as completed: ad_id={}", ad_id);
    Ok(())
}

/// Initiate a purchase session
/// 
/// Creates a payment session with a checkout URL that the user can visit
/// to complete the purchase. The session expires after 30 minutes.
/// 
/// # Arguments
/// * `amount` - Purchase amount (defaults to $15.00 if None)
#[tauri::command]
async fn initiate_purchase(amount: Option<f64>) -> Result<PaymentSession, String> {
    eprintln!("[Monetization] initiate_purchase called: amount={:?}", amount);
    
    let payment_manager = get_payment_manager().await.map_err(|e| {
        eprintln!("[Monetization] Failed to get PaymentManager: {}", e);
        format!("Payment service unavailable: {}", e)
    })?;
    
    match payment_manager.initiate_purchase(None, amount).await {
        Ok(session) => {
            eprintln!("[Monetization] Purchase session initiated: session_id={}", session.session_id);
            Ok(session)
        }
        Err(e) => {
            eprintln!("[Monetization] Failed to initiate purchase: {:?}", e);
            Err(format!("Failed to initiate purchase: {:?}", e))
        }
    }
}

/// Process a payment transaction
/// 
/// Verifies a payment transaction with the payment provider and updates
/// premium status if the payment is successful.
/// 
/// # Arguments
/// * `transaction_id` - Transaction ID from payment provider (e.g., from Paddle webhook)
#[tauri::command]
async fn process_payment(transaction_id: String) -> Result<PurchaseReceipt, String> {
    eprintln!("[Monetization] process_payment called: transaction_id={}", transaction_id);
    
    let premium_service = get_premium_service().await.map_err(|e| {
        eprintln!("[Monetization] Failed to get PremiumService: {}", e);
        format!("Monetization service unavailable: {}", e)
    })?;
    
    let user_id = premium_service.user_id().to_string();
    
    let payment_manager = get_payment_manager().await.map_err(|e| {
        eprintln!("[Monetization] Failed to get PaymentManager: {}", e);
        format!("Payment service unavailable: {}", e)
    })?;
    
    // Process payment
    let premium_status = payment_manager.process_payment(&transaction_id, &user_id).await
        .map_err(|e| {
            eprintln!("[Monetization] Failed to process payment: transaction_id={}, error={:?}", transaction_id, e);
            format!("Failed to process payment: {:?}", e)
        })?;
    
    // Convert PremiumStatus to PurchaseReceipt
    let receipt = PurchaseReceipt {
        transaction_id: premium_status.transaction_id.clone().unwrap_or_default(),
        product_id: "premium".to_string(), // Default product ID
        amount: 15.0, // Default amount, could be extracted from receipt_data
        currency: "USD".to_string(), // Default currency
        status: payment::PaymentStatus::Completed,
        purchase_date: premium_status.purchase_date.to_rfc3339(),
        license_key: premium_status.license_key.clone(),
        customer_email: None,
    };
    
    eprintln!("[Monetization] Payment processed successfully: transaction_id={}", transaction_id);
    Ok(receipt)
}

/// Restore previous purchases
/// 
/// Attempts to restore premium status from local storage and verify it
/// with the payment provider. Returns a list of purchase receipts.
#[tauri::command]
async fn restore_purchases() -> Result<Vec<PurchaseReceipt>, String> {
    eprintln!("[Monetization] restore_purchases called");
    
    let premium_service = get_premium_service().await.map_err(|e| {
        eprintln!("[Monetization] Failed to get PremiumService: {}", e);
        format!("Monetization service unavailable: {}", e)
    })?;
    
    let user_id = premium_service.user_id().to_string();
    
    let payment_manager = get_payment_manager().await.map_err(|e| {
        eprintln!("[Monetization] Failed to get PaymentManager: {}", e);
        format!("Payment service unavailable: {}", e)
    })?;
    
    // Restore purchases
    match payment_manager.restore_purchases(&user_id).await {
        Ok(Some(premium_status)) => {
            // Convert PremiumStatus to PurchaseReceipt
            let receipt = PurchaseReceipt {
                transaction_id: premium_status.transaction_id.clone().unwrap_or_default(),
                product_id: "premium".to_string(),
                amount: 15.0,
                currency: "USD".to_string(),
                status: payment::PaymentStatus::Completed,
                purchase_date: premium_status.purchase_date.to_rfc3339(),
                license_key: premium_status.license_key.clone(),
                customer_email: None,
            };
            
            eprintln!("[Monetization] Purchases restored: transaction_id={}", receipt.transaction_id);
            Ok(vec![receipt])
        }
        Ok(None) => {
            eprintln!("[Monetization] No purchases found to restore");
            Ok(vec![])
        }
        Err(e) => {
            eprintln!("[Monetization] Failed to restore purchases: {:?}", e);
            Err(format!("Failed to restore purchases: {:?}", e))
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_caches,
            get_cache_size,
            clean_cache,
            clean_cache_with_monetization,
            check_chrome_running,
            check_permissions,
            scan_indexed_db_items,
            clean_indexed_db_items,
            scan_large_caches,
            remove_large_caches,
            scan_npm_caches,
            remove_npm_caches,
            // Custom scanner commands
            register_custom_scanner,
            list_custom_scanners,
            remove_custom_scanner,
            scan_custom_caches,
            clean_custom_cache,
            // Smart suggestions commands
            scan_smart_suggestions,
            get_folder_suggestion_info,
            remove_smart_suggestions,
            // Monetization commands
            check_premium_status,
            request_ad,
            ad_completed,
            initiate_purchase,
            process_payment,
            restore_purchases,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
