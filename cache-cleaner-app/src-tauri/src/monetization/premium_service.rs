//! PremiumService for managing premium user status
//!
//! This module provides a high-level service for managing premium user status,
//! including local status checking, granting/revoking premium, and remote verification.
//! It integrates with PaymentManager for receipt validation and PremiumStorage for
//! persistent storage.

use crate::monetization::storage::{PremiumStorage, PremiumStatus, StorageError};
use crate::monetization::payment_manager::{PaymentManager, PaymentManagerError};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Premium service error types
#[derive(Debug, Error)]
pub enum PremiumServiceError {
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),

    #[error("Payment manager error: {0}")]
    PaymentManagerError(#[from] PaymentManagerError),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("Device ID generation failed: {0}")]
    DeviceIdError(String),
}

/// Premium service for managing premium user status
///
/// This service provides a high-level interface for checking and managing
/// premium status, with in-memory caching for performance and integration
/// with PaymentManager for receipt validation.
pub struct PremiumService {
    /// Premium storage manager
    storage: Arc<PremiumStorage>,
    /// Payment manager for receipt validation
    payment_manager: Arc<PaymentManager>,
    /// Cached premium status (for fast access)
    cached_status: Arc<RwLock<Option<PremiumStatus>>>,
    /// Device/user identifier
    user_id: String,
    /// Verification interval (default: 7 days)
    verification_interval: Duration,
}

impl PremiumService {
    /// Create a new PremiumService instance
    ///
    /// # Arguments
    /// * `storage` - Premium storage manager
    /// * `payment_manager` - Payment manager for receipt validation
    /// * `user_id` - User/device identifier (if None, will be generated)
    ///
    /// # Errors
    /// Returns error if device ID generation fails
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use cache_cleaner::monetization::{PremiumService, PremiumStorage, PaymentManager};
    /// use cache_cleaner::payment::PaddleClient;
    ///
    /// let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// let payment_manager = Arc::new(PaymentManager::new(paddle_client, storage.clone()));
    /// let service = PremiumService::new(storage, payment_manager, None).unwrap();
    /// ```
    pub fn new(
        storage: Arc<PremiumStorage>,
        payment_manager: Arc<PaymentManager>,
        user_id: Option<String>,
    ) -> Result<Self, PremiumServiceError> {
        let user_id = user_id.unwrap_or_else(|| Self::generate_device_id().unwrap_or_else(|_| {
            // Fallback to timestamp-based ID if generation fails
            format!("device_{}", Utc::now().timestamp())
        }));

        Ok(Self {
            storage,
            payment_manager,
            cached_status: Arc::new(RwLock::new(None)),
            user_id,
            verification_interval: Duration::days(7),
        })
    }

    /// Create a new PremiumService with custom verification interval
    ///
    /// # Arguments
    /// * `storage` - Premium storage manager
    /// * `payment_manager` - Payment manager for receipt validation
    /// * `user_id` - User/device identifier (if None, will be generated)
    /// * `verification_interval` - How often to verify premium status remotely
    ///
    /// # Errors
    /// Returns error if device ID generation fails
    pub fn with_verification_interval(
        storage: Arc<PremiumStorage>,
        payment_manager: Arc<PaymentManager>,
        user_id: Option<String>,
        verification_interval: Duration,
    ) -> Result<Self, PremiumServiceError> {
        let user_id = user_id.unwrap_or_else(|| Self::generate_device_id().unwrap_or_else(|_| {
            format!("device_{}", Utc::now().timestamp())
        }));

        Ok(Self {
            storage,
            payment_manager,
            cached_status: Arc::new(RwLock::new(None)),
            user_id,
            verification_interval,
        })
    }

    /// Generate a device-specific identifier
    ///
    /// Uses a combination of device identifiers to create a consistent device ID.
    fn generate_device_id() -> Result<String, PremiumServiceError> {
        let mut hasher = Sha256::new();

        // Add home directory path as device identifier
        if let Some(home) = dirs::home_dir() {
            hasher.update(home.to_string_lossy().as_bytes());
            
            // Add username if available (extract from home path)
            if let Some(user) = home.file_name() {
                let user_str = user.to_string_lossy().to_string();
                hasher.update(user_str.as_bytes());
            }
        }

        // Add a constant salt
        hasher.update(b"cache-cleaner-device-id-v1");

        let hash = hasher.finalize();
        let device_id = format!("device_{:x}", hash);
        Ok(device_id)
    }

    /// Get the current user/device ID
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Check if user has premium status (local check)
    ///
    /// This method checks the cached status first, then falls back to
    /// loading from storage. It does NOT perform remote verification.
    ///
    /// # Returns
    /// * `true` if user has premium status
    /// * `false` if user does not have premium status or status is missing/corrupted
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{PremiumService, PremiumStorage, PaymentManager};
    /// # use cache_cleaner::payment::PaddleClient;
    /// # let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// # let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// # let payment_manager = Arc::new(PaymentManager::new(paddle_client, storage.clone()));
    /// # let service = PremiumService::new(storage, payment_manager, None).unwrap();
    /// let is_premium = service.is_premium().await;
    /// ```
    pub async fn is_premium(&self) -> bool {
        // Check cached status first
        {
            let cached = self.cached_status.read().await;
            if let Some(ref status) = *cached {
                // Verify user ID matches
                if status.user_id == self.user_id {
                    return status.is_premium;
                }
            }
        }

        // Load from storage
        match self.load_status().await {
            Ok(Some(status)) => {
                // Verify user ID matches
                if status.user_id == self.user_id {
                    let is_premium = status.is_premium;
                    // Update cache
                    *self.cached_status.write().await = Some(status);
                    return is_premium;
                }
            }
            Ok(None) => {
                // No status found
                return false;
            }
            Err(e) => {
                eprintln!("[PremiumService] Error loading status: {:?}", e);
                return false;
            }
        }

        false
    }

    /// Grant premium status after purchase
    ///
    /// This method should be called after a successful payment transaction.
    /// It validates the receipt with PaymentManager and stores the premium status.
    ///
    /// # Arguments
    /// * `transaction_id` - Transaction ID from payment provider
    ///
    /// # Returns
    /// * `Ok(())` if premium status was granted successfully
    ///
    /// # Errors
    /// * `PremiumServiceError::PaymentManagerError` - If receipt validation fails
    /// * `PremiumServiceError::StorageError` - If storage update fails
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{PremiumService, PremiumStorage, PaymentManager};
    /// # use cache_cleaner::payment::PaddleClient;
    /// # let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// # let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// # let payment_manager = Arc::new(PaymentManager::new(paddle_client, storage.clone()));
    /// # let service = PremiumService::new(storage, payment_manager, None).unwrap();
    /// service.grant_premium("txn_12345").await?;
    /// ```
    pub async fn grant_premium(&self, transaction_id: &str) -> Result<(), PremiumServiceError> {
        eprintln!(
            "[PremiumService] Granting premium: transaction_id={}, user_id={}",
            transaction_id, self.user_id
        );

        // Process payment through PaymentManager (validates receipt and saves status)
        let premium_status = self
            .payment_manager
            .process_payment(transaction_id, &self.user_id)
            .await?;

        // Verify the status was saved correctly
        if premium_status.user_id != self.user_id {
            return Err(PremiumServiceError::InvalidStatus(format!(
                "User ID mismatch: expected {}, got {}",
                self.user_id, premium_status.user_id
            )));
        }

        // Update cache atomically
        *self.cached_status.write().await = Some(premium_status);

        eprintln!(
            "[PremiumService] Premium granted successfully: transaction_id={}, user_id={}",
            transaction_id, self.user_id
        );

        Ok(())
    }

    /// Revoke premium status
    ///
    /// Removes premium status from storage and cache. This should be called
    /// when premium status needs to be revoked (e.g., refund, subscription cancellation).
    ///
    /// # Returns
    /// * `Ok(())` if premium status was revoked successfully
    ///
    /// # Errors
    /// * `PremiumServiceError::StorageError` - If storage update fails
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{PremiumService, PremiumStorage, PaymentManager};
    /// # use cache_cleaner::payment::PaddleClient;
    /// # let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// # let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// # let payment_manager = Arc::new(PaymentManager::new(paddle_client, storage.clone()));
    /// # let service = PremiumService::new(storage, payment_manager, None).unwrap();
    /// service.revoke_premium().await?;
    /// ```
    pub async fn revoke_premium(&self) -> Result<(), PremiumServiceError> {
        eprintln!("[PremiumService] Revoking premium: user_id={}", self.user_id);

        // Load current status
        let mut status = match self.load_status().await? {
            Some(s) => s,
            None => {
                // No status to revoke
                eprintln!("[PremiumService] No premium status to revoke: user_id={}", self.user_id);
                return Ok(());
            }
        };

        // Verify user ID matches
        if status.user_id != self.user_id {
            eprintln!(
                "[PremiumService] User ID mismatch during revoke: expected {}, got {}",
                self.user_id, status.user_id
            );
            return Err(PremiumServiceError::InvalidStatus(format!(
                "User ID mismatch: expected {}, got {}",
                self.user_id, status.user_id
            )));
        }

        // Revoke premium status
        status.is_premium = false;
        status.last_verified = Utc::now();

        // Save to storage
        self.storage.save(&status)?;

        // Update cache atomically
        *self.cached_status.write().await = Some(status);

        eprintln!("[PremiumService] Premium revoked successfully: user_id={}", self.user_id);

        Ok(())
    }

    /// Verify premium status remotely
    ///
    /// Verifies premium status with the payment provider. This should be called
    /// periodically (e.g., on app launch, weekly) to ensure premium status is still valid.
    ///
    /// # Returns
    /// * `Ok(true)` if premium status is valid
    /// * `Ok(false)` if premium status is invalid or missing
    ///
    /// # Errors
    /// * `PremiumServiceError::PaymentManagerError` - If verification fails due to network/API errors
    /// * `PremiumServiceError::StorageError` - If storage operations fail
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{PremiumService, PremiumStorage, PaymentManager};
    /// # use cache_cleaner::payment::PaddleClient;
    /// # let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// # let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// # let payment_manager = Arc::new(PaymentManager::new(paddle_client, storage.clone()));
    /// # let service = PremiumService::new(storage, payment_manager, None).unwrap();
    /// let is_valid = service.verify_premium_status().await?;
    /// ```
    pub async fn verify_premium_status(&self) -> Result<bool, PremiumServiceError> {
        eprintln!("[PremiumService] Verifying premium status: user_id={}", self.user_id);

        // Load current status
        let status = match self.load_status().await? {
            Some(s) => s,
            None => {
                eprintln!("[PremiumService] No premium status to verify: user_id={}", self.user_id);
                return Ok(false);
            }
        };

        // Verify user ID matches
        if status.user_id != self.user_id {
            eprintln!(
                "[PremiumService] User ID mismatch during verification: expected {}, got {}",
                self.user_id, status.user_id
            );
            return Ok(false);
        }

        // If not premium, no need to verify
        if !status.is_premium {
            eprintln!("[PremiumService] User is not premium: user_id={}", self.user_id);
            return Ok(false);
        }

        // Check if verification is needed (based on last_verified timestamp)
        let time_since_verification = Utc::now() - status.last_verified;
        if time_since_verification < self.verification_interval {
            eprintln!(
                "[PremiumService] Verification not needed yet (last verified: {}): user_id={}",
                status.last_verified, self.user_id
            );
            // Status is still valid based on time, but we should still verify if we have a transaction ID
            if status.transaction_id.is_none() {
                return Ok(true); // No transaction to verify, trust local status
            }
        }

        // Verify with payment provider if we have a transaction ID
        if let Some(ref transaction_id) = status.transaction_id {
            match self.payment_manager.validate_receipt(transaction_id).await {
                Ok(_receipt) => {
                    // Receipt is valid, update last_verified timestamp
                    let mut updated_status = status.clone();
                    updated_status.last_verified = Utc::now();

                    // Save updated status
                    self.storage.save(&updated_status)?;

                    // Update cache
                    *self.cached_status.write().await = Some(updated_status);

                    eprintln!(
                        "[PremiumService] Premium status verified successfully: user_id={}, transaction_id={}",
                        self.user_id, transaction_id
                    );

                    return Ok(true);
                }
                Err(e) => {
                    eprintln!(
                        "[PremiumService] Receipt verification failed: user_id={}, transaction_id={}, error={:?}",
                        self.user_id, transaction_id, e
                    );

                    // If verification fails, revoke premium status
                    self.revoke_premium().await?;

                    return Ok(false);
                }
            }
        }

        // If no transaction ID, check expiry date
        if let Some(expiry) = status.expiry_date {
            if expiry < Utc::now() {
                eprintln!(
                    "[PremiumService] Premium status expired: user_id={}, expiry={}",
                    self.user_id, expiry
                );
                self.revoke_premium().await?;
                return Ok(false);
            }
        }

        // No transaction ID and no expiry (lifetime premium), trust local status
        eprintln!(
            "[PremiumService] Premium status valid (lifetime, no verification needed): user_id={}",
            self.user_id
        );
        Ok(true)
    }

    /// Load premium status from storage
    ///
    /// # Returns
    /// * `Ok(Some(PremiumStatus))` - Status loaded successfully
    /// * `Ok(None)` - No status found
    /// * `Err(PremiumServiceError)` - Error loading status
    async fn load_status(&self) -> Result<Option<PremiumStatus>, PremiumServiceError> {
        self.storage.load().map_err(|e| PremiumServiceError::StorageError(e))
    }

    /// Get cached premium status (if available)
    ///
    /// # Returns
    /// * `Some(PremiumStatus)` if status is cached
    /// * `None` if status is not cached
    pub async fn get_cached_status(&self) -> Option<PremiumStatus> {
        self.cached_status.read().await.clone()
    }

    /// Clear cached premium status
    ///
    /// Forces the next `is_premium()` call to reload from storage.
    pub async fn clear_cache(&self) {
        *self.cached_status.write().await = None;
    }

    /// Initialize premium service (load and verify status)
    ///
    /// This should be called on app launch to:
    /// 1. Load premium status from storage
    /// 2. Verify status with payment provider if needed
    /// 3. Update cache
    ///
    /// # Returns
    /// * `Ok(bool)` - Whether user has premium status after initialization
    ///
    /// # Errors
    /// * `PremiumServiceError` - If initialization fails
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{PremiumService, PremiumStorage, PaymentManager};
    /// # use cache_cleaner::payment::PaddleClient;
    /// # let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// # let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// # let payment_manager = Arc::new(PaymentManager::new(paddle_client, storage.clone()));
    /// # let service = PremiumService::new(storage, payment_manager, None).unwrap();
    /// let is_premium = service.initialize().await?;
    /// ```
    pub async fn initialize(&self) -> Result<bool, PremiumServiceError> {
        eprintln!("[PremiumService] Initializing: user_id={}", self.user_id);

        // Try to restore purchases from PaymentManager
        match self.payment_manager.restore_purchases(&self.user_id).await {
            Ok(Some(status)) => {
                // Premium status restored and verified
                *self.cached_status.write().await = Some(status);
                eprintln!(
                    "[PremiumService] Premium status restored: user_id={}",
                    self.user_id
                );
                return Ok(true);
            }
            Ok(None) => {
                // No premium status found
                *self.cached_status.write().await = None;
                eprintln!(
                    "[PremiumService] No premium status found: user_id={}",
                    self.user_id
                );
                return Ok(false);
            }
            Err(e) => {
                eprintln!(
                    "[PremiumService] Error restoring purchases: user_id={}, error={:?}",
                    self.user_id, e
                );
                // Fall back to local status check
            }
        }

        // Fall back to local status check
        let is_premium = self.is_premium().await;
        Ok(is_premium)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payment::config::PaddleConfig;
    use crate::payment::PaddleClient;
    use tempfile::TempDir;

    fn create_test_storage() -> (Arc<PremiumStorage>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("premium_status.json");
        let encryption_key = vec![0u8; 32]; // Test key
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

    fn create_test_service() -> (PremiumService, Arc<PremiumStorage>, TempDir) {
        let (storage, temp_dir) = create_test_storage();
        let paddle_client = create_test_paddle_client();
        let payment_manager = Arc::new(PaymentManager::new(paddle_client, storage.clone()));
        let service = PremiumService::new(storage.clone(), payment_manager, Some("test_user".to_string())).unwrap();
        (service, storage, temp_dir)
    }

    #[tokio::test]
    async fn test_premium_service_creation() {
        let (service, _storage, _temp_dir) = create_test_service();
        assert_eq!(service.user_id(), "test_user");
    }

    #[tokio::test]
    async fn test_is_premium_no_status() {
        let (service, _storage, _temp_dir) = create_test_service();
        let is_premium = service.is_premium().await;
        assert!(!is_premium);
    }

    #[tokio::test]
    async fn test_is_premium_with_status() {
        let (service, storage, _temp_dir) = create_test_service();
        
        // Create a premium status
        let premium_status = PremiumStatus {
            schema_version: 1,
            user_id: "test_user".to_string(),
            is_premium: true,
            purchase_date: Utc::now(),
            expiry_date: None,
            transaction_id: None,
            receipt_data: None,
            last_verified: Utc::now(),
            provider: "paddle".to_string(),
            license_key: None,
        };
        
        // Save to storage
        storage.save(&premium_status).unwrap();
        
        // Check premium status
        let is_premium = service.is_premium().await;
        assert!(is_premium);
    }

    #[tokio::test]
    async fn test_revoke_premium() {
        let (service, storage, _temp_dir) = create_test_service();
        
        // Create a premium status
        let premium_status = PremiumStatus {
            schema_version: 1,
            user_id: "test_user".to_string(),
            is_premium: true,
            purchase_date: Utc::now(),
            expiry_date: None,
            transaction_id: None,
            receipt_data: None,
            last_verified: Utc::now(),
            provider: "paddle".to_string(),
            license_key: None,
        };
        
        // Save to storage
        storage.save(&premium_status).unwrap();
        
        // Verify premium status
        assert!(service.is_premium().await);
        
        // Revoke premium
        service.revoke_premium().await.unwrap();
        
        // Verify premium is revoked
        assert!(!service.is_premium().await);
    }

    #[tokio::test]
    async fn test_revoke_premium_no_status() {
        let (service, _storage, _temp_dir) = create_test_service();
        
        // Revoke when no status exists (should not error)
        let result = service.revoke_premium().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_premium_status_no_status() {
        let (service, _storage, _temp_dir) = create_test_service();
        
        let is_valid = service.verify_premium_status().await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let (service, storage, _temp_dir) = create_test_service();
        
        // Create a premium status
        let premium_status = PremiumStatus {
            schema_version: 1,
            user_id: "test_user".to_string(),
            is_premium: true,
            purchase_date: Utc::now(),
            expiry_date: None,
            transaction_id: None,
            receipt_data: None,
            last_verified: Utc::now(),
            provider: "paddle".to_string(),
            license_key: None,
        };
        
        // Save to storage
        storage.save(&premium_status).unwrap();
        
        // Load status (will cache it)
        service.is_premium().await;
        
        // Verify cache is populated
        assert!(service.get_cached_status().await.is_some());
        
        // Clear cache
        service.clear_cache().await;
        
        // Verify cache is cleared
        assert!(service.get_cached_status().await.is_none());
    }

    #[tokio::test]
    async fn test_initialize_no_status() {
        let (service, _storage, _temp_dir) = create_test_service();
        
        let is_premium = service.initialize().await.unwrap();
        assert!(!is_premium);
    }

    #[tokio::test]
    async fn test_user_id_mismatch() {
        let (service, storage, _temp_dir) = create_test_service();
        
        // Create a premium status for different user
        let premium_status = PremiumStatus {
            schema_version: 1,
            user_id: "other_user".to_string(),
            is_premium: true,
            purchase_date: Utc::now(),
            expiry_date: None,
            transaction_id: None,
            receipt_data: None,
            last_verified: Utc::now(),
            provider: "paddle".to_string(),
            license_key: None,
        };
        
        // Save to storage
        storage.save(&premium_status).unwrap();
        
        // Check premium status (should return false due to user ID mismatch)
        let is_premium = service.is_premium().await;
        assert!(!is_premium);
    }

    #[tokio::test]
    async fn test_generate_device_id() {
        let device_id = PremiumService::generate_device_id().unwrap();
        assert!(device_id.starts_with("device_"));
        assert_eq!(device_id.len(), 71); // "device_" (7 chars) + 64 hex chars = 71
    }
}

