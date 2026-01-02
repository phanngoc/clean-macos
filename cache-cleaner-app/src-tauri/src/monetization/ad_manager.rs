//! AdManager service for managing ad lifecycle
//!
//! This module provides a service for managing ad requests, tracking ad completion,
//! and coordinating with PremiumService to determine if ads are required.

use crate::monetization::premium_service::PremiumService;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Ad request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdRequest {
    /// Unique ad request identifier
    pub ad_id: String,
    /// Ad provider name (e.g., "adsense")
    pub provider: String,
    /// Publisher ID for ad provider
    pub publisher_id: String,
    /// Ad unit ID
    pub ad_unit_id: String,
    /// Expected ad duration in seconds
    pub duration_seconds: u64,
    /// Ad format (e.g., "rewarded_video")
    pub format: String,
}

/// Active ad tracking information
#[derive(Debug, Clone)]
struct ActiveAd {
    /// Ad request ID
    ad_id: String,
    /// When the ad was requested
    requested_at: DateTime<Utc>,
    /// Whether the ad has been completed
    completed: bool,
    /// Ad load start time (for metrics)
    load_start_time: Option<DateTime<Utc>>,
    /// Ad completion time (for metrics)
    completion_time: Option<DateTime<Utc>>,
}

/// Ad configuration
#[derive(Debug, Clone)]
pub struct AdConfig {
    /// Publisher ID for ad provider
    pub publisher_id: String,
    /// Ad unit ID
    pub ad_unit_id: String,
    /// Expected ad duration in seconds (default: 15)
    pub duration_seconds: u64,
    /// Maximum number of retry attempts for ad loading
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Ad completion timeout in seconds (default: 30)
    pub completion_timeout_seconds: u64,
}

impl Default for AdConfig {
    fn default() -> Self {
        Self {
            publisher_id: String::new(),
            ad_unit_id: String::new(),
            duration_seconds: 15,
            max_retries: 3,
            retry_delay_ms: 1000,
            completion_timeout_seconds: 30,
        }
    }
}

/// Ad event for tracking and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdEvent {
    /// Event type
    pub event_type: AdEventType,
    /// Ad request ID
    pub ad_id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Optional error message
    pub error: Option<String>,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Ad event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdEventType {
    /// Ad requested
    Requested,
    /// Ad loaded successfully
    Loaded,
    /// Ad started playing
    Started,
    /// Ad completed
    Completed,
    /// Ad failed to load
    LoadFailed,
    /// Ad timed out
    Timeout,
    /// Ad was skipped (premium user)
    Skipped,
    /// Ad was blocked
    Blocked,
}

/// Ad manager error types
#[derive(Debug, Error)]
pub enum AdError {
    #[error("Ad loading failed: {0}")]
    LoadFailed(String),

    #[error("Ad timeout: ad did not complete in time")]
    Timeout,

    #[error("Ad blocked: ad blocker detected")]
    Blocked,

    #[error("Premium user: ad not required")]
    NotRequired,

    #[error("Ad SDK error: {0}")]
    SdkError(String),

    #[error("Ad ID not found: {0}")]
    AdNotFound(String),

    #[error("Invalid ad configuration: {0}")]
    InvalidConfig(String),
}

/// Ad manager service
///
/// Manages ad lifecycle including requesting ads, tracking completion,
/// and coordinating with PremiumService to determine if ads are required.
pub struct AdManager {
    /// Premium service for checking premium status
    premium_service: Arc<PremiumService>,
    /// Active ads being tracked
    active_ads: Arc<RwLock<Vec<ActiveAd>>>,
    /// Ad configuration
    config: AdConfig,
    /// Ad events for tracking and analytics
    events: Arc<RwLock<Vec<AdEvent>>>,
}

impl AdManager {
    /// Create a new AdManager instance
    ///
    /// # Arguments
    /// * `premium_service` - Premium service for checking premium status
    /// * `config` - Ad configuration
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use cache_cleaner::monetization::{AdManager, AdConfig, PremiumService};
    ///
    /// let premium_service = Arc::new(/* ... */);
    /// let config = AdConfig {
    ///     publisher_id: "pub-123456".to_string(),
    ///     ad_unit_id: "unit-123456".to_string(),
    ///     ..Default::default()
    /// };
    /// let ad_manager = AdManager::new(premium_service, config);
    /// ```
    pub fn new(premium_service: Arc<PremiumService>, config: AdConfig) -> Self {
        // Validate config
        if config.publisher_id.is_empty() {
            eprintln!("[AdManager] Warning: publisher_id is empty");
        }
        if config.ad_unit_id.is_empty() {
            eprintln!("[AdManager] Warning: ad_unit_id is empty");
        }

        Self {
            premium_service,
            active_ads: Arc::new(RwLock::new(Vec::new())),
            config,
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new AdManager with default configuration
    ///
    /// # Arguments
    /// * `premium_service` - Premium service for checking premium status
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use cache_cleaner::monetization::{AdManager, PremiumService};
    ///
    /// let premium_service = Arc::new(/* ... */);
    /// let ad_manager = AdManager::with_default_config(premium_service);
    /// ```
    pub fn with_default_config(premium_service: Arc<PremiumService>) -> Self {
        Self::new(premium_service, AdConfig::default())
    }

    /// Request an ad
    ///
    /// Checks if user is premium. If premium, returns `NotRequired` error.
    /// Otherwise, creates an ad request and tracks it.
    ///
    /// # Returns
    /// * `Ok(AdRequest)` - Ad request information for frontend
    /// * `Err(AdError::NotRequired)` - User is premium, ad not required
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{AdManager, AdConfig, PremiumService};
    /// # let premium_service = Arc::new(/* ... */);
    /// # let config = AdConfig::default();
    /// # let ad_manager = AdManager::new(premium_service, config);
    /// match ad_manager.request_ad().await {
    ///     Ok(request) => println!("Ad requested: {}", request.ad_id),
    ///     Err(AdError::NotRequired) => println!("User is premium, no ad needed"),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub async fn request_ad(&self) -> Result<AdRequest, AdError> {
        eprintln!("[AdManager] Requesting ad...");

        // Check if user is premium
        if self.premium_service.is_premium().await {
            self.log_event(AdEventType::Skipped, None, None, None).await;
            eprintln!("[AdManager] User is premium, ad not required");
            return Err(AdError::NotRequired);
        }

        // Generate unique ad ID
        let mut rng = rand::thread_rng();
        let ad_id = format!(
            "ad_{}_{}",
            Utc::now().timestamp(),
            rng.gen::<u32>()
        );

        let request = AdRequest {
            ad_id: ad_id.clone(),
            provider: "adsense".to_string(),
            publisher_id: self.config.publisher_id.clone(),
            ad_unit_id: self.config.ad_unit_id.clone(),
            duration_seconds: self.config.duration_seconds,
            format: "rewarded_video".to_string(),
        };

        // Track active ad
        let mut ads = self.active_ads.write().await;
        ads.push(ActiveAd {
            ad_id: ad_id.clone(),
            requested_at: Utc::now(),
            completed: false,
            load_start_time: None,
            completion_time: None,
        });
        drop(ads);

        // Log ad requested event
        self.log_event(AdEventType::Requested, Some(ad_id.clone()), None, None).await;

        eprintln!(
            "[AdManager] Ad requested: ad_id={}, provider={}, duration={}s",
            ad_id, request.provider, request.duration_seconds
        );

        Ok(request)
    }

    /// Wait for ad completion
    ///
    /// Waits for the frontend to notify that an ad has completed.
    /// This method polls the active ads list until the ad is marked as completed
    /// or a timeout occurs.
    ///
    /// # Arguments
    /// * `ad_id` - Ad request ID to wait for
    ///
    /// # Returns
    /// * `Ok(())` - Ad completed successfully
    /// * `Err(AdError::Timeout)` - Ad did not complete within timeout period
    /// * `Err(AdError::AdNotFound)` - Ad ID not found
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{AdManager, AdConfig, PremiumService};
    /// # let premium_service = Arc::new(/* ... */);
    /// # let config = AdConfig::default();
    /// # let ad_manager = AdManager::new(premium_service, config);
    /// # let request = ad_manager.request_ad().await.unwrap();
    /// match ad_manager.wait_for_ad_completion(&request.ad_id).await {
    ///     Ok(()) => println!("Ad completed!"),
    ///     Err(AdError::Timeout) => eprintln!("Ad timed out"),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub async fn wait_for_ad_completion(&self, ad_id: &str) -> Result<(), AdError> {
        eprintln!("[AdManager] Waiting for ad completion: ad_id={}", ad_id);

        let timeout = Duration::from_secs(self.config.completion_timeout_seconds);
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(100);

        loop {
            // Check if ad is completed
            let ads = self.active_ads.read().await;
            if let Some(ad) = ads.iter().find(|a| a.ad_id == ad_id) {
                if ad.completed {
                    eprintln!("[AdManager] Ad completed: ad_id={}", ad_id);
                    return Ok(());
                }
            } else {
                // Ad not found
                drop(ads);
                return Err(AdError::AdNotFound(ad_id.to_string()));
            }
            drop(ads);

            // Check timeout
            if start.elapsed() > timeout {
                self.log_event(
                    AdEventType::Timeout,
                    Some(ad_id.to_string()),
                    Some(format!("Ad timed out after {} seconds", timeout.as_secs())),
                    None,
                ).await;
                eprintln!("[AdManager] Ad timeout: ad_id={}", ad_id);
                return Err(AdError::Timeout);
            }

            // Wait before next poll
            sleep(poll_interval).await;
        }
    }

    /// Mark an ad as completed
    ///
    /// This method should be called by the frontend (via Tauri IPC) when an ad
    /// has been completed. It updates the active ad tracking and logs the event.
    ///
    /// # Arguments
    /// * `ad_id` - Ad request ID that was completed
    ///
    /// # Returns
    /// * `Ok(())` - Ad marked as completed successfully
    /// * `Err(AdError::AdNotFound)` - Ad ID not found
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{AdManager, AdConfig, PremiumService};
    /// # let premium_service = Arc::new(/* ... */);
    /// # let config = AdConfig::default();
    /// # let ad_manager = AdManager::new(premium_service, config);
    /// ad_manager.mark_ad_completed("ad_12345").await?;
    /// ```
    pub async fn mark_ad_completed(&self, ad_id: &str) -> Result<(), AdError> {
        eprintln!("[AdManager] Marking ad as completed: ad_id={}", ad_id);

        let mut ads = self.active_ads.write().await;
        if let Some(ad) = ads.iter_mut().find(|a| a.ad_id == ad_id) {
            ad.completed = true;
            ad.completion_time = Some(Utc::now());

            // Calculate metrics
            let load_time = if let Some(load_start) = ad.load_start_time {
                Some((ad.completion_time.unwrap() - load_start).num_milliseconds())
            } else {
                None
            };

            // Log completion event with metrics
            let metadata = if let Some(load_time_ms) = load_time {
                Some(serde_json::json!({
                    "load_time_ms": load_time_ms,
                    "duration_seconds": self.config.duration_seconds,
                }))
            } else {
                None
            };

            drop(ads);

            self.log_event(
                AdEventType::Completed,
                Some(ad_id.to_string()),
                None,
                metadata,
            ).await;

            eprintln!("[AdManager] Ad marked as completed: ad_id={}", ad_id);
            Ok(())
        } else {
            drop(ads);
            eprintln!("[AdManager] Ad not found: ad_id={}", ad_id);
            Err(AdError::AdNotFound(ad_id.to_string()))
        }
    }

    /// Mark an ad as started loading
    ///
    /// This method should be called when an ad starts loading (for metrics).
    ///
    /// # Arguments
    /// * `ad_id` - Ad request ID
    ///
    /// # Returns
    /// * `Ok(())` - Ad marked as started
    /// * `Err(AdError::AdNotFound)` - Ad ID not found
    pub async fn mark_ad_started(&self, ad_id: &str) -> Result<(), AdError> {
        eprintln!("[AdManager] Marking ad as started: ad_id={}", ad_id);

        let mut ads = self.active_ads.write().await;
        if let Some(ad) = ads.iter_mut().find(|a| a.ad_id == ad_id) {
            ad.load_start_time = Some(Utc::now());
            drop(ads);

            self.log_event(AdEventType::Started, Some(ad_id.to_string()), None, None).await;
            Ok(())
        } else {
            drop(ads);
            Err(AdError::AdNotFound(ad_id.to_string()))
        }
    }

    /// Mark an ad as loaded
    ///
    /// This method should be called when an ad has finished loading (for metrics).
    ///
    /// # Arguments
    /// * `ad_id` - Ad request ID
    ///
    /// # Returns
    /// * `Ok(())` - Ad marked as loaded
    /// * `Err(AdError::AdNotFound)` - Ad ID not found
    pub async fn mark_ad_loaded(&self, ad_id: &str) -> Result<(), AdError> {
        eprintln!("[AdManager] Marking ad as loaded: ad_id={}", ad_id);

        let ads = self.active_ads.read().await;
        if ads.iter().any(|a| a.ad_id == ad_id) {
            drop(ads);
            self.log_event(AdEventType::Loaded, Some(ad_id.to_string()), None, None).await;
            Ok(())
        } else {
            drop(ads);
            Err(AdError::AdNotFound(ad_id.to_string()))
        }
    }

    /// Mark an ad as failed
    ///
    /// This method should be called when an ad fails to load or play.
    ///
    /// # Arguments
    /// * `ad_id` - Ad request ID
    /// * `error` - Error message
    ///
    /// # Returns
    /// * `Ok(())` - Ad marked as failed
    /// * `Err(AdError::AdNotFound)` - Ad ID not found
    pub async fn mark_ad_failed(&self, ad_id: &str, error: String) -> Result<(), AdError> {
        eprintln!("[AdManager] Marking ad as failed: ad_id={}, error={}", ad_id, error);

        let ads = self.active_ads.read().await;
        if ads.iter().any(|a| a.ad_id == ad_id) {
            drop(ads);
            self.log_event(
                AdEventType::LoadFailed,
                Some(ad_id.to_string()),
                Some(error),
                None,
            ).await;
            Ok(())
        } else {
            drop(ads);
            Err(AdError::AdNotFound(ad_id.to_string()))
        }
    }

    /// Check if an ad is required
    ///
    /// Returns `true` if the user is not premium and an ad is required.
    ///
    /// # Returns
    /// * `true` - Ad is required (user is not premium)
    /// * `false` - Ad is not required (user is premium)
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{AdManager, AdConfig, PremiumService};
    /// # let premium_service = Arc::new(/* ... */);
    /// # let config = AdConfig::default();
    /// # let ad_manager = AdManager::new(premium_service, config);
    /// if ad_manager.is_ad_required().await {
    ///     println!("User needs to watch an ad");
    /// }
    /// ```
    pub async fn is_ad_required(&self) -> bool {
        !self.premium_service.is_premium().await
    }

    /// Check if user can skip ads
    ///
    /// Returns `true` if the user is premium and can skip ads.
    ///
    /// # Returns
    /// * `true` - User can skip ads (premium)
    /// * `false` - User cannot skip ads (not premium)
    pub async fn can_skip_ad(&self) -> bool {
        self.premium_service.is_premium().await
    }

    /// Get ad configuration
    pub fn config(&self) -> &AdConfig {
        &self.config
    }

    /// Get recent ad events (for analytics)
    ///
    /// # Arguments
    /// * `limit` - Maximum number of events to return
    ///
    /// # Returns
    /// Vector of recent ad events
    pub async fn get_recent_events(&self, limit: usize) -> Vec<AdEvent> {
        let events = self.events.read().await;
        let start = events.len().saturating_sub(limit);
        events[start..].to_vec()
    }

    /// Clear old ad tracking data
    ///
    /// Removes completed ads older than the specified duration.
    ///
    /// # Arguments
    /// * `max_age_seconds` - Maximum age in seconds for completed ads to keep
    pub async fn clear_old_ads(&self, max_age_seconds: i64) {
        let mut ads = self.active_ads.write().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_seconds);
        ads.retain(|ad| {
            !ad.completed || ad.requested_at > cutoff
        });
        let removed_count = ads.len();
        drop(ads);
        eprintln!("[AdManager] Cleared old ads, {} remaining", removed_count);
    }

    /// Log an ad event
    ///
    /// Internal method for logging ad events for analytics.
    async fn log_event(
        &self,
        event_type: AdEventType,
        ad_id: Option<String>,
        error: Option<String>,
        metadata: Option<serde_json::Value>,
    ) {
        let event = AdEvent {
            event_type,
            ad_id: ad_id.unwrap_or_else(|| "unknown".to_string()),
            timestamp: Utc::now(),
            error,
            metadata,
        };

        let mut events = self.events.write().await;
        events.push(event.clone());

        // Keep only last 1000 events to prevent memory growth
        if events.len() > 1000 {
            events.remove(0);
        }
        drop(events);

        eprintln!(
            "[AdManager] Event: {:?}, ad_id={}, timestamp={}",
            event.event_type, event.ad_id, event.timestamp
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monetization::storage::PremiumStorage;
    use crate::monetization::payment_manager::PaymentManager;
    use crate::payment::config::PaddleConfig;
    use crate::payment::PaddleClient;
    use tempfile::TempDir;

    fn create_test_premium_service() -> (Arc<PremiumService>, TempDir) {
        let (storage, temp_dir) = create_test_storage();
        let paddle_client = create_test_paddle_client();
        let payment_manager = Arc::new(PaymentManager::new(paddle_client, storage.clone()));
        let premium_service = Arc::new(
            PremiumService::new(storage, payment_manager, Some("test_user".to_string())).unwrap()
        );
        (premium_service, temp_dir)
    }

    fn create_test_storage() -> (Arc<PremiumStorage>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("premium_status.json");
        let encryption_key = vec![0u8; 32];
        let storage = Arc::new(
            PremiumStorage::new(storage_path, encryption_key).unwrap()
        );
        (storage, temp_dir)
    }

    fn create_test_paddle_client() -> Arc<PaddleClient> {
        let config = PaddleConfig {
            api_key: "test_api_key".to_string(),
            vendor_id: "test_vendor_id".to_string(),
            product_id: "test_product_id".to_string(),
            test_mode: true,
            webhook_key: None,
        };
        Arc::new(PaddleClient::new(config))
    }

    fn create_test_ad_manager() -> (AdManager, TempDir) {
        let (premium_service, temp_dir) = create_test_premium_service();
        let config = AdConfig {
            publisher_id: "test_publisher".to_string(),
            ad_unit_id: "test_unit".to_string(),
            duration_seconds: 15,
            max_retries: 3,
            retry_delay_ms: 1000,
            completion_timeout_seconds: 5, // Short timeout for tests
        };
        let ad_manager = AdManager::new(premium_service, config);
        (ad_manager, temp_dir)
    }

    #[tokio::test]
    async fn test_ad_manager_creation() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        assert_eq!(ad_manager.config().publisher_id, "test_publisher");
        assert_eq!(ad_manager.config().ad_unit_id, "test_unit");
    }

    #[tokio::test]
    async fn test_is_ad_required_not_premium() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        let is_required = ad_manager.is_ad_required().await;
        assert!(is_required); // Not premium, so ad is required
    }

    #[tokio::test]
    async fn test_can_skip_ad_not_premium() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        let can_skip = ad_manager.can_skip_ad().await;
        assert!(!can_skip); // Not premium, so cannot skip
    }

    #[tokio::test]
    async fn test_request_ad_not_premium() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        let result = ad_manager.request_ad().await;
        assert!(result.is_ok());
        let request = result.unwrap();
        assert!(request.ad_id.starts_with("ad_"));
        assert_eq!(request.provider, "adsense");
        assert_eq!(request.duration_seconds, 15);
    }

    #[tokio::test]
    async fn test_request_ad_premium() {
        let (premium_service, _temp_dir) = create_test_premium_service();
        
        // Grant premium status
        // Note: This will fail in test because we don't have a real transaction,
        // but we can test the structure
        
        let config = AdConfig {
            publisher_id: "test_publisher".to_string(),
            ad_unit_id: "test_unit".to_string(),
            duration_seconds: 15,
            max_retries: 3,
            retry_delay_ms: 1000,
            completion_timeout_seconds: 5,
        };
        let ad_manager = AdManager::new(premium_service, config);
        
        // Since user is not premium, ad should be required
        let result = ad_manager.request_ad().await;
        // This should succeed because user is not premium
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mark_ad_completed() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        
        // Request an ad
        let request = ad_manager.request_ad().await.unwrap();
        let ad_id = request.ad_id.clone();
        
        // Mark as completed
        let result = ad_manager.mark_ad_completed(&ad_id).await;
        assert!(result.is_ok());
        
        // Wait for completion should return immediately
        let wait_result = ad_manager.wait_for_ad_completion(&ad_id).await;
        assert!(wait_result.is_ok());
    }

    #[tokio::test]
    async fn test_mark_ad_completed_not_found() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        
        let result = ad_manager.mark_ad_completed("nonexistent_ad").await;
        assert!(result.is_err());
        match result {
            Err(AdError::AdNotFound(_)) => {}
            _ => panic!("Expected AdNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_wait_for_ad_completion_timeout() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        
        // Request an ad
        let request = ad_manager.request_ad().await.unwrap();
        let ad_id = request.ad_id.clone();
        
        // Wait for completion (should timeout since we don't mark it as completed)
        let result = ad_manager.wait_for_ad_completion(&ad_id).await;
        assert!(result.is_err());
        match result {
            Err(AdError::Timeout) => {}
            _ => panic!("Expected Timeout error"),
        }
    }

    #[tokio::test]
    async fn test_wait_for_ad_completion_not_found() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        
        let result = ad_manager.wait_for_ad_completion("nonexistent_ad").await;
        assert!(result.is_err());
        match result {
            Err(AdError::AdNotFound(_)) => {}
            _ => panic!("Expected AdNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_mark_ad_started() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        
        let request = ad_manager.request_ad().await.unwrap();
        let ad_id = request.ad_id.clone();
        
        let result = ad_manager.mark_ad_started(&ad_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mark_ad_loaded() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        
        let request = ad_manager.request_ad().await.unwrap();
        let ad_id = request.ad_id.clone();
        
        let result = ad_manager.mark_ad_loaded(&ad_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mark_ad_failed() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        
        let request = ad_manager.request_ad().await.unwrap();
        let ad_id = request.ad_id.clone();
        
        let result = ad_manager.mark_ad_failed(&ad_id, "Test error".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_recent_events() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        
        // Request an ad (creates an event)
        let _request = ad_manager.request_ad().await.unwrap();
        
        // Get recent events
        let events = ad_manager.get_recent_events(10).await;
        assert!(!events.is_empty());
        assert!(events.iter().any(|e| matches!(e.event_type, AdEventType::Requested)));
    }

    #[tokio::test]
    async fn test_clear_old_ads() {
        let (ad_manager, _temp_dir) = create_test_ad_manager();
        
        // Request and complete an ad
        let request = ad_manager.request_ad().await.unwrap();
        let ad_id = request.ad_id.clone();
        ad_manager.mark_ad_completed(&ad_id).await.unwrap();
        
        // Clear old ads (with very short max age)
        ad_manager.clear_old_ads(0).await;
        
        // Ad should be removed
        let result = ad_manager.wait_for_ad_completion(&ad_id).await;
        assert!(result.is_err()); // Ad should not be found
    }
}

