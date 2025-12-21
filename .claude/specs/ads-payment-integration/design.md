# Technical Design: Ads & Payment Integration

## Overview

This document describes the technical architecture for integrating advertising and payment systems into the Cache Cleaner macOS application. The design implements a dual-monetization model: free users watch 15-second ads before deletions, while premium users ($15 one-time purchase) skip all ads.

The architecture is built on Tauri (Rust backend + frontend), requiring careful integration of native macOS payment systems and web-based ad SDKs.

**Selected Providers** (based on Task #1 and #2 research):
- **Ad Provider**: Google AdSense via web integration (best Tauri compatibility)
- **Payment Provider**: Paddle for direct distribution (primary), Apple IAP for App Store (if applicable)

**Integration Points**:
- Existing `clean_cache()` command in `main.rs` will be wrapped with monetization logic
- New monetization modules will be added: `src/monetization/` directory
- Frontend components will integrate with existing UI structure

## Architecture

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Web UI)                        │
│  - Delete Button Handlers                                   │
│  - Ad Display Component                                     │
│  - Payment UI Component                                     │
│  - Premium Status Indicator                                 │
└────────────────┬────────────────────────────────────────────┘
                 │ Tauri IPC
┌────────────────▼────────────────────────────────────────────┐
│              Tauri Backend (Rust)                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Monetization Service Layer                   │  │
│  │  - AdManager: Ad display & completion tracking       │  │
│  │  - PaymentManager: Purchase processing               │  │
│  │  - PremiumService: Status management & verification │  │
│  └────────────┬───────────────────────┬───────────────────┘  │
│               │                     │                       │
│  ┌────────────▼──────────┐  ┌───────▼──────────────┐        │
│  │   Local Storage      │  │   Cache Cleaner     │        │
│  │   - Premium status   │  │   - Deletion logic  │        │
│  │   - Purchase receipt │  │   - Cache scanning │        │
│  └──────────────────────┘  └─────────────────────┘        │
└────────────────┬───────────────────────┬───────────────────┘
                 │                       │
    ┌────────────▼──────────┐  ┌─────────▼──────────────┐
    │   Google AdSense      │  │  Paddle API            │
    │   (Web-based SDK)     │  │  (REST API)            │
    │   - Ad loading        │  │  - Transaction proc.  │
    │   - Ad playback       │  │  - Receipt validation │
    │   - Completion events │  │  - Status verification│
    │   - Rewarded videos   │  │  - License keys        │
    └───────────────────────┘  └───────────────────────┘
```

### Component Breakdown

#### Component 1: AdManager (Rust Backend)
- **Purpose**: Manages ad lifecycle, loading, display, and completion tracking
- **Location**: `src/monetization/ad_manager.rs`
- **Responsibilities**: 
  - Initialize ad SDK in frontend context (Google AdSense)
  - Request ad content from provider
  - Track ad playback duration (15 seconds)
  - Verify ad completion before allowing deletion
  - Handle ad loading failures and retries
  - Emit events to frontend for UI updates
  - Coordinate with PremiumService to check if ads are required
- **Dependencies**: Tauri IPC, Google AdSense SDK (via frontend), PremiumService, LocalStorage
- **Interfaces**: 
  ```rust
  use serde::{Deserialize, Serialize};
  use std::sync::Arc;
  use tokio::sync::RwLock;
  
  pub struct AdManager {
      premium_service: Arc<PremiumService>,
      active_ads: Arc<RwLock<Vec<ActiveAd>>>,
      config: AdConfig,
  }
  
  #[derive(Clone, Serialize, Deserialize)]
  pub struct AdRequest {
      pub ad_id: String,
      pub provider: String, // "adsense"
      pub publisher_id: String, // Google AdSense publisher ID
      pub ad_unit_id: String, // AdSense ad unit ID
      pub duration_seconds: u64, // 15
      pub format: String, // "rewarded_video"
  }
  
  #[derive(Clone)]
  struct ActiveAd {
      ad_id: String,
      requested_at: chrono::DateTime<chrono::Utc>,
      completed: bool,
  }
  
  #[derive(Clone)]
  pub struct AdConfig {
      pub publisher_id: String,
      pub ad_unit_id: String,
      pub duration_seconds: u64,
      pub max_retries: u32,
      pub retry_delay_ms: u64,
  }
  
  #[derive(Debug, thiserror::Error)]
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
  }
  
  impl AdManager {
      pub fn new(premium_service: Arc<PremiumService>, config: AdConfig) -> Self {
          Self {
              premium_service,
              active_ads: Arc::new(RwLock::new(Vec::new())),
              config,
          }
      }
      
      pub async fn request_ad(&self) -> Result<AdRequest, AdError> {
          // Check if user is premium
          if self.premium_service.is_premium().await {
              return Err(AdError::NotRequired);
          }
          
          let ad_id = format!("ad_{}", uuid::Uuid::new_v4());
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
              ad_id,
              requested_at: chrono::Utc::now(),
              completed: false,
          });
          
          Ok(request)
      }
      
      pub async fn wait_for_ad_completion(&self, ad_id: String) -> Result<(), AdError> {
          // Wait for frontend to notify completion via Tauri IPC
          // This is handled by the ad_completed command
          let timeout = tokio::time::Duration::from_secs(30);
          let start = std::time::Instant::now();
          
          loop {
              let ads = self.active_ads.read().await;
              if let Some(ad) = ads.iter().find(|a| a.ad_id == ad_id) {
                  if ad.completed {
                      return Ok(());
                  }
              }
              
              if start.elapsed() > timeout {
                  return Err(AdError::Timeout);
              }
              
              drop(ads);
              tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
          }
      }
      
      pub async fn mark_ad_completed(&self, ad_id: String) -> Result<(), AdError> {
          let mut ads = self.active_ads.write().await;
          if let Some(ad) = ads.iter_mut().find(|a| a.ad_id == ad_id) {
              ad.completed = true;
              Ok(())
          } else {
              Err(AdError::LoadFailed("Ad ID not found".to_string()))
          }
      }
      
      pub async fn is_ad_required(&self) -> bool {
          !self.premium_service.is_premium().await
      }
      
      pub async fn can_skip_ad(&self) -> bool {
          self.premium_service.is_premium().await
      }
  }
  ```

#### Component 2: PaymentManager (Rust Backend)
- **Purpose**: Handles premium purchase transactions and receipt validation
- **Location**: `src/monetization/payment_manager.rs`
- **Responsibilities**: 
  - Initiate payment flow with Paddle provider
  - Process payment transactions via Paddle API
  - Validate purchase receipts with Paddle
  - Update premium status in local storage
  - Verify premium status on app startup
  - Handle payment failures and retries
  - Support multiple payment providers (Paddle primary, Apple IAP for App Store)
- **Dependencies**: Paddle REST API (reqwest), LocalStorage, PremiumService, encryption utilities
- **Interfaces**:
  ```rust
  use serde::{Deserialize, Serialize};
  use std::sync::Arc;
  use reqwest::Client;
  
  pub struct PaymentManager {
      client: Client,
      paddle_config: PaddleConfig,
      premium_service: Arc<PremiumService>,
  }
  
  #[derive(Clone)]
  pub struct PaddleConfig {
      pub vendor_id: String,
      pub api_key: String,
      pub product_id: String, // Paddle product ID for premium license
      pub price: f64, // 15.00
      pub currency: String, // "USD"
      pub environment: String, // "sandbox" or "production"
  }
  
  #[derive(Clone, Serialize, Deserialize)]
  pub struct PaymentSession {
      pub session_id: String,
      pub checkout_url: String, // Paddle hosted checkout URL
      pub expires_at: chrono::DateTime<chrono::Utc>,
  }
  
  #[derive(Clone, Serialize, Deserialize)]
  pub struct PurchaseReceipt {
      pub transaction_id: String,
      pub receipt_id: String, // Paddle subscription_id or order_id
      pub purchase_date: chrono::DateTime<chrono::Utc>,
      pub amount: f64,
      pub currency: String,
      pub status: String, // "completed", "pending", "failed"
      pub provider: String, // "paddle" or "apple_iap"
      pub license_key: Option<String>, // Paddle license key
      pub receipt_data: String, // Encrypted receipt data
  }
  
  #[derive(Debug, thiserror::Error)]
  pub enum PaymentError {
      #[error("Network error: {0}")]
      NetworkError(String),
      #[error("Payment declined: {0}")]
      Declined(String),
      #[error("Invalid receipt: {0}")]
      InvalidReceipt(String),
      #[error("Payment timeout")]
      Timeout,
      #[error("Paddle API error: {0}")]
      PaddleApiError(String),
      #[error("Configuration error: {0}")]
      ConfigError(String),
  }
  
  impl PaymentManager {
      pub fn new(paddle_config: PaddleConfig, premium_service: Arc<PremiumService>) -> Self {
          Self {
              client: Client::new(),
              paddle_config,
              premium_service,
          }
      }
      
      pub async fn initiate_purchase(&self, amount: f64) -> Result<PaymentSession, PaymentError> {
          // Create Paddle checkout session
          let checkout_url = format!(
              "https://checkout.paddle.com/product/{}?vendor_id={}&price={}&currency={}",
              self.paddle_config.product_id,
              self.paddle_config.vendor_id,
              amount,
              self.paddle_config.currency
          );
          
          let session_id = uuid::Uuid::new_v4().to_string();
          let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
          
          Ok(PaymentSession {
              session_id,
              checkout_url,
              expires_at,
          })
      }
      
      pub async fn process_paddle_webhook(
          &self,
          webhook_data: PaddleWebhook,
      ) -> Result<PurchaseReceipt, PaymentError> {
          // Process Paddle webhook notification
          // Validate webhook signature
          // Extract transaction details
          // Create receipt
          
          let receipt = PurchaseReceipt {
              transaction_id: webhook_data.transaction_id.clone(),
              receipt_id: webhook_data.subscription_id.clone(),
              purchase_date: chrono::Utc::now(),
              amount: webhook_data.amount,
              currency: webhook_data.currency,
              status: "completed".to_string(),
              provider: "paddle".to_string(),
              license_key: webhook_data.license_key.clone(),
              receipt_data: serde_json::to_string(&webhook_data)
                  .map_err(|e| PaymentError::InvalidReceipt(e.to_string()))?,
          };
          
          // Grant premium status
          self.premium_service
              .grant_premium(receipt.clone())
              .await
              .map_err(|e| PaymentError::PaddleApiError(e.to_string()))?;
          
          Ok(receipt)
      }
      
      pub async fn validate_receipt(
          &self,
          receipt: &PurchaseReceipt,
      ) -> Result<bool, PaymentError> {
          match receipt.provider.as_str() {
              "paddle" => self.validate_paddle_receipt(receipt).await,
              "apple_iap" => self.validate_apple_receipt(receipt).await,
              _ => Err(PaymentError::InvalidReceipt("Unknown provider".to_string())),
          }
      }
      
      async fn validate_paddle_receipt(
          &self,
          receipt: &PurchaseReceipt,
      ) -> Result<bool, PaymentError> {
          // Call Paddle API to verify license key
          let url = format!(
              "https://vendors.paddle.com/api/2.0/product/validate_license",
          );
          
          let response = self
              .client
              .post(&url)
              .form(&[
                  ("vendor_id", &self.paddle_config.vendor_id),
                  ("vendor_auth_code", &self.paddle_config.api_key),
                  ("product_id", &self.paddle_config.product_id),
                  ("license_code", receipt.license_key.as_ref().unwrap()),
              ])
              .send()
              .await
              .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
          
          let result: PaddleValidationResponse = response
              .json()
              .await
              .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
          
          Ok(result.valid)
      }
      
      async fn validate_apple_receipt(
          &self,
          _receipt: &PurchaseReceipt,
      ) -> Result<bool, PaymentError> {
          // Apple IAP validation (if App Store distribution)
          // Implementation for App Store receipt validation
          Ok(false) // Placeholder
      }
      
      pub async fn restore_purchases(&self) -> Result<Vec<PurchaseReceipt>, PaymentError> {
          // Load from local storage
          // Verify each receipt with provider
          // Return valid receipts
          Ok(vec![])
      }
  }
  
  #[derive(Deserialize)]
  struct PaddleWebhook {
      transaction_id: String,
      subscription_id: String,
      amount: f64,
      currency: String,
      license_key: Option<String>,
  }
  
  #[derive(Deserialize)]
  struct PaddleValidationResponse {
      valid: bool,
  }
  ```

#### Component 3: PremiumService (Rust Backend)
- **Purpose**: Manages premium user status, persistence, and verification
- **Location**: `src/monetization/premium_service.rs`
- **Responsibilities**: 
  - Store and retrieve premium status from encrypted local storage
  - Encrypt premium status data using AES-256
  - Verify premium status on app launch
  - Sync premium status with payment provider
  - Handle premium status expiration (lifetime premium, no expiration)
  - Cache premium status in memory for fast access
- **Dependencies**: LocalStorage (encrypted JSON file), PaymentManager, encryption utilities (aes-gcm)
- **Interfaces**:
  ```rust
  use serde::{Deserialize, Serialize};
  use std::sync::Arc;
  use std::path::PathBuf;
  use tokio::sync::RwLock;
  
  pub struct PremiumService {
      storage_path: PathBuf,
      payment_manager: Arc<PaymentManager>,
      cached_status: Arc<RwLock<Option<PremiumStatus>>>,
      encryption_key: Vec<u8>, // AES-256 key
  }
  
  #[derive(Clone, Serialize, Deserialize)]
  pub struct PremiumStatus {
      pub user_id: String, // Device identifier
      pub is_premium: bool,
      pub purchase_date: chrono::DateTime<chrono::Utc>,
      pub expiry_date: Option<chrono::DateTime<chrono::Utc>>, // None for lifetime
      pub transaction_id: String,
      pub receipt_data: String, // Encrypted receipt
      pub last_verified: chrono::DateTime<chrono::Utc>,
      pub provider: String, // "paddle" or "apple_iap"
  }
  
  #[derive(Debug, thiserror::Error)]
  pub enum PremiumError {
      #[error("Storage error: {0}")]
      StorageError(String),
      #[error("Encryption error: {0}")]
      EncryptionError(String),
      #[error("Verification failed: {0}")]
      VerificationFailed(String),
      #[error("Invalid receipt: {0}")]
      InvalidReceipt(String),
  }
  
  impl PremiumService {
      pub fn new(
          storage_path: PathBuf,
          payment_manager: Arc<PaymentManager>,
          encryption_key: Vec<u8>,
      ) -> Self {
          Self {
              storage_path,
              payment_manager,
              cached_status: Arc::new(RwLock::new(None)),
              encryption_key,
          }
      }
      
      pub async fn is_premium(&self) -> bool {
          // Check cached status first
          {
              let cached = self.cached_status.read().await;
              if let Some(ref status) = *cached {
                  return status.is_premium;
              }
          }
          
          // Load from storage
          if let Ok(status) = self.load_status().await {
              let is_premium = status.is_premium;
              *self.cached_status.write().await = Some(status);
              return is_premium;
          }
          
          false
      }
      
      pub async fn grant_premium(
          &self,
          receipt: PurchaseReceipt,
      ) -> Result<(), PremiumError> {
          // Validate receipt first
          let is_valid = self
              .payment_manager
              .validate_receipt(&receipt)
              .await
              .map_err(|e| PremiumError::VerificationFailed(e.to_string()))?;
          
          if !is_valid {
              return Err(PremiumError::InvalidReceipt("Receipt validation failed".to_string()));
          }
          
          let device_id = self.get_device_id();
          let status = PremiumStatus {
              user_id: device_id,
              is_premium: true,
              purchase_date: receipt.purchase_date,
              expiry_date: None, // Lifetime premium
              transaction_id: receipt.transaction_id,
              receipt_data: self.encrypt_data(&receipt.receipt_data)?,
              last_verified: chrono::Utc::now(),
              provider: receipt.provider,
          };
          
          // Save to storage
          self.save_status(&status).await?;
          
          // Update cache
          *self.cached_status.write().await = Some(status);
          
          Ok(())
      }
      
      pub async fn revoke_premium(&self) -> Result<(), PremiumError> {
          let mut status = self.load_status().await?;
          status.is_premium = false;
          self.save_status(&status).await?;
          *self.cached_status.write().await = Some(status);
          Ok(())
      }
      
      pub async fn verify_premium_status(&self) -> Result<bool, PremiumError> {
          let status = self.load_status().await?;
          
          if !status.is_premium {
              return Ok(false);
          }
          
          // Verify with payment provider
          let receipt = PurchaseReceipt {
              transaction_id: status.transaction_id.clone(),
              receipt_id: "".to_string(),
              purchase_date: status.purchase_date,
              amount: 15.0,
              currency: "USD".to_string(),
              status: "completed".to_string(),
              provider: status.provider.clone(),
              license_key: None,
              receipt_data: self.decrypt_data(&status.receipt_data)?,
          };
          
          let is_valid = self
              .payment_manager
              .validate_receipt(&receipt)
              .await
              .map_err(|e| PremiumError::VerificationFailed(e.to_string()))?;
          
          if !is_valid {
              // Revoke premium if verification fails
              self.revoke_premium().await?;
              return Ok(false);
          }
          
          // Update last verified timestamp
          let mut updated_status = status;
          updated_status.last_verified = chrono::Utc::now();
          self.save_status(&updated_status).await?;
          
          Ok(true)
      }
      
      async fn load_status(&self) -> Result<PremiumStatus, PremiumError> {
          // Load and decrypt from storage
          // Implementation details...
          todo!()
      }
      
      async fn save_status(&self, status: &PremiumStatus) -> Result<(), PremiumError> {
          // Encrypt and save to storage
          // Implementation details...
          todo!()
      }
      
      fn encrypt_data(&self, data: &str) -> Result<String, PremiumError> {
          // AES-256-GCM encryption
          // Implementation details...
          todo!()
      }
      
      fn decrypt_data(&self, encrypted: &str) -> Result<String, PremiumError> {
          // AES-256-GCM decryption
          // Implementation details...
          todo!()
      }
      
      fn get_device_id(&self) -> String {
          // Generate or retrieve device identifier
          // Implementation details...
          todo!()
      }
  }
  ```

#### Component 4: AdDisplayComponent (Frontend)
- **Purpose**: UI component for displaying advertisements
- **Responsibilities**: 
  - Render ad container
  - Load ad content from provider SDK
  - Display ad playback with timer
  - Prevent ad dismissal before 15 seconds
  - Emit completion events to backend
  - Handle ad loading errors
- **Dependencies**: Ad provider SDK, Tauri IPC, React/Vue component framework
- **Interfaces**:
  ```typescript
  interface AdDisplayProps {
      onComplete: () => void;
      onError: (error: AdError) => void;
      duration: number; // 15 seconds
  }
  
  function AdDisplay({ onComplete, onError, duration }: AdDisplayProps): JSX.Element;
  ```

#### Component 5: PaymentComponent (Frontend)
- **Purpose**: UI component for premium purchase flow
- **Responsibilities**: 
  - Display premium purchase options
  - Show payment form or native payment UI
  - Handle payment submission
  - Display payment status (processing, success, error)
  - Show purchase confirmation
- **Dependencies**: Payment provider SDK, Tauri IPC, React/Vue component framework
- **Interfaces**:
  ```typescript
  interface PaymentComponentProps {
      price: number; // $15
      onSuccess: (receipt: PurchaseReceipt) => void;
      onError: (error: PaymentError) => void;
  }
  
  function PaymentComponent({ price, onSuccess, onError }: PaymentComponentProps): JSX.Element;
  ```

#### Component 6: MonetizationMiddleware (Rust Backend)
- **Purpose**: Intercepts deletion requests and enforces ad/payment requirements
- **Location**: `src/main.rs` (modified `clean_cache` command)
- **Responsibilities**: 
  - Check premium status before deletion
  - Trigger ad display for free users via frontend
  - Block deletion until ad completes (via Tauri IPC)
  - Allow immediate deletion for premium users
  - Log monetization events
  - Integrate with existing `cache::cleaner::clean()` function
- **Dependencies**: PremiumService, AdManager, existing Cache Cleaner module
- **Integration Point**: Wraps existing `clean_cache` Tauri command
- **Interfaces**:
  ```rust
  use crate::cache::{CacheType, CleanResult};
  use crate::monetization::{AdManager, PremiumService};
  use std::sync::Arc;
  
  pub struct MonetizationMiddleware {
      ad_manager: Arc<AdManager>,
      premium_service: Arc<PremiumService>,
  }
  
  impl MonetizationMiddleware {
      pub fn new(
          ad_manager: Arc<AdManager>,
          premium_service: Arc<PremiumService>,
      ) -> Self {
          Self {
              ad_manager,
              premium_service,
          }
      }
      
  pub async fn clean_cache_with_monetization(
          &self,
          cache_type: CacheType,
      dry_run: bool,
  ) -> Result<CleanResult, MonetizationError> {
      // Check premium status
          let is_premium = self.premium_service.is_premium().await;
          
          if !is_premium {
              // Request ad from AdManager
              let ad_request = self.ad_manager.request_ad().await
                  .map_err(|e| MonetizationError::AdError(e))?;
              
              // Frontend will display ad and call ad_completed when done
              // We wait for completion here
              self.ad_manager.wait_for_ad_completion(ad_request.ad_id).await
                  .map_err(|e| MonetizationError::AdError(e))?;
          }
          
          // Proceed with deletion (existing cache cleaner logic)
          cache::cleaner::clean(&cache_type, dry_run).await
              .map_err(|e| MonetizationError::CleanError(e.to_string()))
      }
  }
  
  #[derive(Debug, thiserror::Error)]
  pub enum MonetizationError {
      #[error("Ad error: {0}")]
      AdError(crate::monetization::AdError),
      #[error("Clean error: {0}")]
      CleanError(String),
  }
  
  // Modified Tauri command in main.rs
  #[tauri::command]
  async fn clean_cache(
      cache_type: String,
      dry_run: bool,
      state: tauri::State<'_, MonetizationMiddleware>,
  ) -> Result<CleanResult, String> {
      let ct = CacheType::from_str(&cache_type)
          .map_err(|e| e.to_string())?;
      
      state.clean_cache_with_monetization(ct, dry_run).await
          .map_err(|e| e.to_string())
  }
  ```

## Data Model

### Entities

```
Entity: PremiumStatus
Fields:
  - user_id: String (device identifier or account ID)
  - is_premium: bool (premium status flag)
  - purchase_date: DateTime<Utc> (when premium was purchased)
  - expiry_date: Option<DateTime<Utc>> (None for lifetime premium)
  - transaction_id: String (payment provider transaction ID)
  - receipt_data: String (encrypted purchase receipt)
  - last_verified: DateTime<Utc> (last verification timestamp)
Relationships:
  - One-to-one with User/Device
  - References PaymentTransaction

Entity: PaymentTransaction
Fields:
  - transaction_id: String (unique transaction identifier)
  - amount: f64 ($15.00)
  - currency: String ("USD")
  - status: TransactionStatus (pending, completed, failed, refunded)
  - payment_method: String (credit_card, apple_pay, etc.)
  - provider: String (stripe, paddle, etc.)
  - created_at: DateTime<Utc>
  - completed_at: Option<DateTime<Utc>>
  - receipt_url: Option<String> (receipt download URL)
Relationships:
  - One-to-one with PremiumStatus

Entity: AdEvent
Fields:
  - event_id: String (unique event identifier)
  - ad_provider: String (admob, unity, etc.)
  - ad_unit_id: String (ad placement identifier)
  - event_type: AdEventType (loaded, started, completed, failed, skipped)
  - duration: Option<u64> (actual playback duration in seconds)
  - timestamp: DateTime<Utc>
  - user_id: String (for analytics)
Relationships:
  - Many-to-one with User/Device (for analytics)
```

### Data Flow

1. **Ad Display Flow**:
   - User clicks delete → Frontend checks premium status via Tauri command
   - If not premium → Frontend requests ad from AdManager
   - AdManager initializes ad SDK → Ad loads in frontend component
   - Ad plays for 15 seconds → Frontend tracks playback duration
   - Ad completes → Frontend notifies AdManager → AdManager allows deletion
   - Deletion proceeds → Cache cleaner executes

2. **Premium Purchase Flow**:
   - User clicks "Upgrade to Premium" → Frontend shows PaymentComponent
   - User enters payment info → PaymentComponent submits to PaymentManager
   - PaymentManager processes with provider → Provider returns transaction result
   - PaymentManager validates receipt → PremiumService grants premium status
   - PremiumService stores encrypted status → Frontend updates UI
   - Future deletions skip ads

3. **Premium Status Verification Flow**:
   - App launches → PremiumService loads status from local storage
   - PremiumService verifies status with PaymentManager
   - PaymentManager validates receipt with provider API
   - If valid → Premium status confirmed
   - If invalid → Premium status revoked, user reverted to free tier

## Sequence Diagrams

### Primary Flow: Free User Deletion with Ad

```
User → Frontend: Click "Delete Cache"
Frontend → Tauri Backend: check_premium_status()
Tauri Backend → PremiumService: is_premium()
PremiumService → LocalStorage: read_premium_status()
LocalStorage → PremiumService: false (not premium)
PremiumService → Tauri Backend: false
Tauri Backend → Frontend: requires_ad = true
Frontend → AdDisplayComponent: Show ad
AdDisplayComponent → AdProvider SDK: Load ad
AdProvider SDK → AdDisplayComponent: Ad content
AdDisplayComponent → User: Display ad (15 seconds)
AdDisplayComponent → Tauri Backend: ad_completed()
Tauri Backend → AdManager: record_ad_completion()
AdManager → Tauri Backend: allow_deletion()
Tauri Backend → Cache Cleaner: clean_cache()
Cache Cleaner → Tauri Backend: CleanResult
Tauri Backend → Frontend: Deletion complete
Frontend → User: Show success message
```

### Alternative Flow: Premium User Deletion (No Ad)

```
User → Frontend: Click "Delete Cache"
Frontend → Tauri Backend: check_premium_status()
Tauri Backend → PremiumService: is_premium()
PremiumService → LocalStorage: read_premium_status()
LocalStorage → PremiumService: true (premium)
PremiumService → Tauri Backend: true
Tauri Backend → Frontend: requires_ad = false
Frontend → Tauri Backend: clean_cache() (immediate)
Tauri Backend → Cache Cleaner: clean_cache()
Cache Cleaner → Tauri Backend: CleanResult
Tauri Backend → Frontend: Deletion complete
Frontend → User: Show success message
```

### Premium Purchase Flow

```
User → Frontend: Click "Upgrade to Premium"
Frontend → PaymentComponent: Show payment form
User → PaymentComponent: Enter payment details
PaymentComponent → Tauri Backend: initiate_purchase(15.00)
Tauri Backend → PaymentManager: process_payment()
PaymentManager → PaymentProvider API: Create payment session
PaymentProvider API → PaymentManager: Payment session ID
PaymentManager → PaymentProvider API: Process transaction
PaymentProvider API → PaymentManager: Transaction result + receipt
PaymentManager → PaymentManager: validate_receipt()
PaymentManager → PaymentProvider API: Verify receipt
PaymentProvider API → PaymentManager: Receipt valid
PaymentManager → PremiumService: grant_premium(receipt)
PremiumService → LocalStorage: Store encrypted premium status
PremiumService → Tauri Backend: Premium granted
Tauri Backend → Frontend: Purchase successful
Frontend → User: Show confirmation, update UI
```

## API Contracts

### Tauri Commands

#### Command 1: check_premium_status
- **Method**: Tauri command (synchronous)
- **Handler**: `check_premium_status() -> Result<bool, String>`
- **Request**: None
- **Response**:
```rust
Ok(true)  // User is premium
Ok(false) // User is not premium
Err(String) // Error checking status
```
- **Error Codes**: N/A (returns Result)

#### Command 2: request_ad
- **Method**: Tauri command (async)
- **Handler**: `request_ad() -> Result<AdRequest, String>`
- **Request**: None
- **Response**:
```rust
pub struct AdRequest {
    pub ad_id: String,
    pub provider: String,
    pub ad_url: Option<String>, // For web-based ads
    pub duration_seconds: u64,  // 15
}
```
- **Error Codes**: Returns error string on failure

#### Command 3: ad_completed
- **Method**: Tauri command (async)
- **Handler**: `ad_completed(ad_id: String) -> Result<(), String>`
- **Request**:
```rust
{
    "ad_id": "ad_12345"
}
```
- **Response**: `Ok(())` on success
- **Error Codes**: Returns error string if ad completion invalid

#### Command 4: initiate_purchase
- **Method**: Tauri command (async)
- **Handler**: `initiate_purchase(amount: f64) -> Result<PaymentSession, String>`
- **Request**:
```rust
{
    "amount": 15.00
}
```
- **Response**:
```rust
pub struct PaymentSession {
    pub session_id: String,
    pub payment_url: Option<String>, // For web-based payments
    pub client_secret: Option<String>, // For Stripe, etc.
}
```
- **Error Codes**: Returns error string on failure

#### Command 5: process_payment
- **Method**: Tauri command (async)
- **Handler**: `process_payment(session_id: String, payment_data: PaymentData) -> Result<PurchaseReceipt, String>`
- **Request**:
```rust
pub struct PaymentData {
    pub payment_method_id: Option<String>,
    pub card_token: Option<String>,
    // Provider-specific payment data
}
```
- **Response**:
```rust
pub struct PurchaseReceipt {
    pub transaction_id: String,
    pub receipt_id: String,
    pub purchase_date: DateTime<Utc>,
    pub amount: f64,
    pub status: String,
}
```
- **Error Codes**: Returns error string on failure

#### Command 6: restore_purchases
- **Method**: Tauri command (async)
- **Handler**: `restore_purchases() -> Result<Vec<PurchaseReceipt>, String>`
- **Request**: None
- **Response**: Vector of purchase receipts
- **Error Codes**: Returns error string on failure

#### Command 7: clean_cache_with_monetization (Modified)
- **Method**: Tauri command (async) - Modified existing command
- **Handler**: `clean_cache_with_monetization(cache_type: String, dry_run: bool) -> Result<CleanResult, String>`
- **Request**: Same as existing `clean_cache`
- **Response**: Same as existing `clean_cache`
- **Behavior**: 
  - Checks premium status
  - If not premium: waits for ad completion (frontend handles ad display)
  - If premium: proceeds immediately
  - Then executes deletion

## Technical Decisions

### Decision 1: Ad SDK Integration Approach
- **Context**: Need to integrate ads into Tauri desktop app
- **Options Considered**:
  1. **Native macOS Ad SDK**: Use macOS-specific ad frameworks
     - Pros: Native performance, better integration
     - Cons: Limited options, may not support desktop apps well
  2. **Web-based Ad SDK**: Use web ad SDKs (Google AdMob, Unity Ads web)
     - Pros: More options, better documentation, cross-platform
     - Cons: Requires web view, may have performance overhead
  3. **Custom Ad Server**: Build own ad serving system
     - Pros: Full control, no revenue share
     - Cons: High development cost, need ad inventory
- **Decision**: Web-based Ad SDK (Option 2)
- **Rationale**: 
  - Tauri already uses web technologies in frontend
  - More ad network options available
  - Better revenue potential
  - Easier integration with existing frontend
- **Trade-offs**: 
  - Slight performance overhead from web view
  - Requires internet connection for ads
  - May need to handle ad blocking

### Decision 2: Payment Provider Selection
- **Context**: Need secure payment processing for $15 one-time purchases
- **Options Considered**:
  1. **Stripe**: Popular payment processor
     - Pros: Excellent API, good documentation, supports one-time payments, lower fees (2.9% + $0.30)
     - Cons: Requires more setup work, less desktop-focused
  2. **Paddle**: Built for software/SaaS
     - Pros: Designed for software sales, handles taxes automatically, excellent desktop app support, license key system
     - Cons: Higher fees (~5% + $0.50), less brand recognition
  3. **Apple In-App Purchase**: Native macOS payment
     - Pros: Native integration, trusted by users, required for App Store
     - Cons: Requires App Store distribution, 30% revenue share, strict guidelines
  4. **RevenueCat**: Payment abstraction layer
     - Pros: Supports multiple providers, receipt validation
     - Cons: Additional abstraction layer, may add complexity
- **Decision**: **Paddle for direct distribution** (primary), **Apple IAP for App Store** (if applicable)
- **Rationale** (based on Task #2 research): 
  - **Paddle selected** because:
    - Desktop-first approach with excellent macOS support
    - Built-in tax handling (reduces complexity)
    - License key system for validation
    - Better suited for software sales than Stripe
    - REST API integration works well with Rust/Tauri
  - **Apple IAP** will be supported if App Store distribution is chosen
  - Hybrid approach allows maximum distribution flexibility
- **Trade-offs**: 
  - Paddle: Higher fees (~5% vs 2.9%) but significantly less setup complexity
  - Automatic tax handling saves development time
  - License key system simplifies validation
  - Apple IAP: Only works in App Store, 30% cut, but required for App Store distribution

### Decision 3: Premium Status Storage
- **Context**: Need to persist premium status locally and verify remotely
- **Options Considered**:
  1. **Encrypted Local File**: Store in encrypted JSON file
     - Pros: Simple, works offline, fast access
     - Cons: Can be tampered with (mitigated by remote verification)
  2. **Keychain (macOS)**: Use macOS Keychain
     - Pros: OS-level security, encrypted by system
     - Cons: More complex API, platform-specific
  3. **SQLite Database**: Store in local database
     - Pros: Structured, queryable, can store history
     - Cons: Overkill for simple boolean flag
  4. **Cloud Sync**: Store on server, sync on launch
     - Pros: Cannot be tampered with, works across devices
     - Cons: Requires account system, always needs internet
- **Decision**: Encrypted Local File + Remote Verification (Hybrid)
- **Rationale**: 
  - Fast local access for UI decisions
  - Periodic remote verification for security
  - Works offline (cached status)
  - Balance between security and UX
- **Trade-offs**: 
  - Local storage can be tampered with, but remote verification catches it
  - Requires internet for initial verification, but works offline after

### Decision 4: Ad Display Timing
- **Context**: When to show ads relative to deletion action
- **Options Considered**:
  1. **Before Deletion**: Show ad, then delete
     - Pros: User sees value before action, clear cause-effect
     - Cons: User must wait before seeing results
  2. **After Deletion**: Delete first, show ad after
     - Pros: Immediate gratification
     - Cons: User might close app, ad not guaranteed to be seen
  3. **During Deletion**: Show ad while deletion happens
     - Pros: No additional wait time
     - Cons: Deletion might finish before ad, complex timing
- **Decision**: Before Deletion (Option 1)
- **Rationale**: 
  - Ensures ad is viewed before value is delivered
  - Clear user expectation: watch ad to unlock deletion
  - Prevents users from skipping ad after deletion
- **Trade-offs**: 
  - Slight delay before deletion, but ensures ad revenue

## Error Handling Strategy

### Ad Loading Errors
- **Retry Logic**: Up to 3 retry attempts with exponential backoff (1s, 2s, 4s)
- **Fallback**: If ad fails to load after 3 attempts, show warning and allow deletion (or require premium)
- **Logging**: Log all ad errors for analytics and debugging
- **User Message**: "Ad failed to load. Please try again or upgrade to Premium."
- **Ad Blocker Detection**: Detect if ad blocker is active, show message: "Please disable ad blocker or upgrade to Premium"

### Payment Processing Errors
- **Network Errors**: Retry with exponential backoff, show "Payment processing, please wait"
- **Payment Declined**: Show clear error message, allow retry with different payment method
- **Timeout**: Show pending state, verify in background, notify when complete
- **Invalid Receipt**: Attempt receipt validation retry, if fails show "Restore Purchase" option
- **Paddle API Errors**: Handle specific Paddle error codes, provide user-friendly messages
- **Logging**: Log all payment errors (without sensitive data) for support

### Premium Status Errors
- **Corrupted Local Storage**: Attempt remote verification, if fails show "Restore Purchase"
- **Verification Failure**: Show warning, allow manual restore, temporarily grant premium if recent purchase
- **Network Offline**: Use cached premium status, verify when online
- **Encryption Errors**: Fallback to unencrypted storage with warning, prompt user to restore

### Error Handling Flowcharts

#### Ad Loading Error Flow
```
User clicks delete
    ↓
Check premium status
    ↓
[Not Premium] → Request ad
    ↓
Ad loading...
    ↓
[Success] → Display ad → Wait 15s → Complete → Delete
    ↓
[Failure] → Retry (attempt 1)
    ↓
[Failure] → Retry (attempt 2)
    ↓
[Failure] → Retry (attempt 3)
    ↓
[Still Failure] → Show error message
    ↓
Options:
  - "Try Again" → Retry from start
  - "Upgrade to Premium" → Show payment UI
  - "Skip Ad" (if allowed) → Delete immediately
```

#### Payment Processing Error Flow
```
User initiates purchase
    ↓
Create payment session
    ↓
[Success] → Open Paddle checkout
    ↓
User completes payment
    ↓
Paddle webhook received
    ↓
[Success] → Validate receipt
    ↓
[Valid] → Grant premium → Success
    ↓
[Invalid] → Retry validation (3x)
    ↓
[Still Invalid] → Show error: "Payment received but verification failed"
    ↓
Options:
  - "Contact Support" → Open support form
  - "Restore Purchase" → Manual restore flow
```

#### Premium Status Verification Error Flow
```
App launches
    ↓
Load premium status from storage
    ↓
[Success] → Check if premium
    ↓
[Is Premium] → Verify with payment provider
    ↓
[Network Error] → Use cached status, verify in background
    ↓
[Verification Success] → Update last_verified → Continue
    ↓
[Verification Failed] → Revoke premium → Show message
    ↓
Options:
  - "Restore Purchase" → Restore flow
  - "Contact Support" → Support form
```

## Security Considerations

### Payment Security
- **PCI Compliance**: Never store payment data locally, all processing through PCI-compliant provider (Paddle)
- **Encryption**: Encrypt premium status and receipts in local storage using AES-256-GCM
- **Receipt Validation**: Always validate receipts with Paddle API before granting premium
- **Transaction Verification**: Verify transaction IDs with Paddle API on app launch
- **Webhook Security**: Validate Paddle webhook signatures to prevent spoofing
- **HTTPS Only**: All API communication over HTTPS with certificate pinning (optional)

### Ad Security
- **Ad SDK Validation**: Use Google AdSense (reputable provider with security best practices)
- **Content Security Policy (CSP)**: Implement CSP headers to prevent XSS attacks from ad content
- **Content Security**: Validate ad content source, prevent malicious ads
- **Privacy Compliance**: Comply with GDPR, CCPA, and other privacy regulations
  - Implement consent management platform (CMP)
  - Show privacy consent banner before ads
  - Store consent preferences locally
- **Ad Blocking Detection**: Detect ad blockers, show message: "Please disable ad blocker or upgrade to Premium"

### Premium Status Security
- **Encryption**: Encrypt premium status data using AES-256-GCM
  - Encryption key derived from device ID + app secret
  - Store encrypted data in `~/.cache-cleaner/premium_status.enc`
- **Tampering Detection**: 
  - Periodic verification with Paddle API (on app launch, weekly)
  - Verify receipt signatures
  - Check transaction status
- **Device Binding**: Bind premium to device ID (prevents sharing across devices)
  - Device ID: Hardware UUID or machine identifier
- **Secure Storage**: Use encrypted JSON file, not plain text
- **Key Management**: Store encryption key securely (not in code, use environment variables or keychain)

### Security Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Security Layers                           │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Application Layer                                   │   │
│  │  - Input validation                                  │   │
│  │  - Rate limiting                                     │   │
│  │  - Error sanitization                                │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ↓                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Encryption Layer                                    │   │
│  │  - AES-256-GCM for premium status                   │   │
│  │  - Encrypted receipt storage                        │   │
│  │  - Key derivation from device ID                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ↓                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Network Security                                    │   │
│  │  - HTTPS/TLS 1.3 for all API calls                  │   │
│  │  - Certificate pinning (optional)                   │   │
│  │  - Webhook signature validation                      │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ↓                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Provider Security                                   │   │
│  │  - Paddle: PCI-DSS compliant                         │   │
│  │  - Google AdSense: Secure ad delivery                │   │
│  │  - No payment data stored locally                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
└─────────────────────────────────────────────────────────────┘

Security Flow:
1. User data → Input validation → Encryption → Secure storage
2. API calls → HTTPS → Signature validation → Provider API
3. Premium status → Encrypted storage → Periodic verification → Provider API
4. Ad content → CSP headers → Sandboxed iframe → Ad provider
```

## Performance Considerations

### Expected Load
- **Concurrent Users**: Support 1000+ concurrent users
- **Ad Requests**: ~10,000 ad requests per day (estimated)
- **Payment Transactions**: ~100 transactions per day (estimated 10% conversion)
- **Status Checks**: ~50,000 status checks per day (on app launch, before deletions)

### Optimization Strategy
- **Caching**: 
  - Cache premium status in memory (RwLock<Option<PremiumStatus>>)
  - Only check storage on app launch and when status changes
  - In-memory cache for fast UI decisions (< 1ms)
- **Ad Preloading**: 
  - Preload next ad while current ad plays (if applicable)
  - Cache ad content temporarily to reduce load times
- **Lazy Loading**: 
  - Load Google AdSense SDK only when needed (not on app startup)
  - Load payment UI components only when user clicks "Upgrade"
- **Background Verification**: 
  - Verify premium status in background on app launch
  - Don't block UI while verification happens
  - Use cached status immediately, update when verification completes
- **Async Operations**:
  - All I/O operations are async (tokio)
  - Non-blocking ad loading and payment processing
  - Parallel operations where possible (e.g., verify multiple receipts)
- **Storage Optimization**:
  - Use efficient serialization (serde_json)
  - Compress encrypted data if needed
  - Clean up old ad events periodically

### Database Indexing
- N/A (using file-based storage, not database)

## Risks & Mitigations

### Risk 1: Ad Provider Unavailability
- **Impact**: High - Revenue loss, users cannot delete
- **Probability**: Medium - Ad networks can have outages
- **Mitigation**: 
  - Support multiple ad providers with fallback
  - Allow deletion after timeout (with warning) or require premium
  - Cache ad content when possible

### Risk 2: Payment Provider Outage
- **Impact**: High - Cannot process premium purchases
- **Probability**: Low - Major providers have high uptime
- **Mitigation**: 
  - Support multiple payment providers
  - Queue payment requests for retry
  - Show clear error messages to users

### Risk 3: Premium Status Tampering
- **Impact**: Medium - Revenue loss from unauthorized premium access
- **Probability**: Medium - Local storage can be modified
- **Mitigation**: 
  - Encrypt premium status
  - Periodic remote verification
  - Device binding (optional)
  - Log all premium status changes

### Risk 4: Ad Blockers
- **Impact**: Medium - Revenue loss from blocked ads
- **Probability**: High - Many users have ad blockers
- **Mitigation**: 
  - Detect ad blockers
  - Show message: "Please disable ad blocker or upgrade to Premium"
  - Consider server-side ad serving (more complex)

### Risk 5: Low Ad Revenue
- **Impact**: High - Feature may not be profitable
- **Probability**: Medium - Desktop ad rates vary
- **Mitigation**: 
  - Research ad networks with best desktop CPM rates
  - Optimize ad placement and timing
  - Focus on premium conversion

### Risk 6: User Friction from Ads
- **Impact**: Medium - Users may abandon app
- **Probability**: Medium - 15-second ads can be annoying
- **Mitigation**: 
  - Make premium price reasonable ($15)
  - Show value proposition clearly
  - Ensure ads are relevant and not too intrusive
  - Consider shorter ads or ad-free credits

## Integration Points with Existing Codebase

### File Structure
```
cache-cleaner-app/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                    # Modified: Add monetization commands
│   │   ├── cache/                     # Existing: No changes needed
│   │   │   ├── mod.rs
│   │   │   ├── cleaner.rs            # Used by monetization middleware
│   │   │   └── ...
│   │   └── monetization/              # NEW: Monetization modules
│   │       ├── mod.rs                 # Module exports
│   │       ├── ad_manager.rs          # AdManager implementation
│   │       ├── payment_manager.rs      # PaymentManager implementation
│   │       ├── premium_service.rs      # PremiumService implementation
│   │       └── storage.rs             # Encrypted storage utilities
│   └── Cargo.toml                     # Modified: Add dependencies
└── ui/
    ├── index.html                     # Modified: Add ad/payment components
    └── components/                    # NEW: Frontend components
        ├── AdDisplay.js               # Ad display component
        └── PaymentComponent.js        # Payment UI component
```

### Modified Files

#### 1. `src/main.rs`
**Changes**:
- Add monetization module: `mod monetization;`
- Initialize monetization services in `main()`
- Modify `clean_cache` command to use `MonetizationMiddleware`
- Add new Tauri commands:
  - `check_premium_status()`
  - `request_ad()`
  - `ad_completed(ad_id)`
  - `initiate_purchase(amount)`
  - `process_paddle_webhook(webhook_data)`
  - `restore_purchases()`

**Integration Pattern**:
```rust
// Existing command (modified)
#[tauri::command]
async fn clean_cache(
    cache_type: String,
    dry_run: bool,
    state: tauri::State<'_, MonetizationMiddleware>,
) -> Result<CleanResult, String> {
    // MonetizationMiddleware wraps existing cache::cleaner::clean()
    state.clean_cache_with_monetization(ct, dry_run).await
}

// Existing cache cleaner (unchanged)
// cache::cleaner::clean() remains the same
```

#### 2. `Cargo.toml`
**New Dependencies**:
```toml
[dependencies]
# Existing dependencies...
# Monetization dependencies
reqwest = { version = "0.11", features = ["json"] }
aes-gcm = "0.10"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

#### 3. Frontend (`ui/index.html` or React/Vue components)
**Changes**:
- Add Google AdSense SDK script tag
- Add AdDisplay component
- Add PaymentComponent (Paddle checkout integration)
- Modify delete button handlers to check premium status
- Add premium status indicator in UI

### Integration Flow

#### Existing Cache Clean Flow (Modified)
```
User clicks delete
    ↓
Frontend: check_premium_status() [NEW]
    ↓
Backend: PremiumService.is_premium() [NEW]
    ↓
[Not Premium] → Frontend: request_ad() [NEW]
    ↓
Frontend: Display ad (Google AdSense) [NEW]
    ↓
Ad completes → Frontend: ad_completed(ad_id) [NEW]
    ↓
Backend: MonetizationMiddleware.clean_cache_with_monetization() [NEW]
    ↓
Backend: cache::cleaner::clean() [EXISTING - unchanged]
    ↓
Return CleanResult [EXISTING - unchanged]
```

#### New Payment Flow
```
User clicks "Upgrade to Premium"
    ↓
Frontend: initiate_purchase(15.00) [NEW]
    ↓
Backend: PaymentManager.initiate_purchase() [NEW]
    ↓
Frontend: Open Paddle checkout URL [NEW]
    ↓
User completes payment
    ↓
Paddle webhook → Backend: process_paddle_webhook() [NEW]
    ↓
Backend: PaymentManager.validate_receipt() [NEW]
    ↓
Backend: PremiumService.grant_premium() [NEW]
    ↓
Frontend: Update UI (show premium badge) [NEW]
```

### Backward Compatibility
- **Existing `clean_cache` command**: Still works, but now wrapped with monetization
- **Existing cache types**: All existing cache types work unchanged
- **Existing UI**: Minimal changes, only add premium indicators
- **Data migration**: No migration needed (new feature)

### Testing Integration Points
1. **Unit Tests**: Test monetization modules independently
2. **Integration Tests**: Test monetization + cache cleaner integration
3. **E2E Tests**: Test full flow from UI click to cache deletion
4. **Mock Tests**: Mock Paddle API and AdSense SDK for testing

## Future Considerations

### Potential Enhancements
- **Subscription Model**: Add monthly/yearly subscription option alongside one-time payment
- **Ad-Free Credits**: Allow users to earn ad-free deletions through other actions
- **Family Sharing**: Share premium status across devices (requires account system)
- **Tiered Pricing**: Multiple premium tiers with different features
- **In-App Currency**: Virtual currency system for more flexible monetization

### Extensibility Points
- **Ad Provider Abstraction**: Design ad interface to support multiple providers easily
  - Trait-based design: `trait AdProvider` with implementations for AdSense, Unity Ads, etc.
- **Payment Provider Abstraction**: Design payment interface to support multiple providers
  - Trait-based design: `trait PaymentProvider` with implementations for Paddle, Stripe, Apple IAP
- **Monetization Strategy**: Make it easy to change monetization model (ads, payments, subscriptions)
  - Strategy pattern for different monetization models

### Technical Debt
- **Ad SDK Updates**: Ad SDKs may require updates, need maintenance plan
- **Payment Provider Changes**: Payment providers may change APIs, need versioning strategy
- **Premium Status Migration**: If changing storage format, need migration path
- **Dependency Updates**: Keep Rust dependencies updated (reqwest, aes-gcm, etc.)

