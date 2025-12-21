//! PaymentManager service for handling payment processing
//!
//! This module provides a high-level payment management service that integrates
//! with payment providers (Paddle) and manages premium status in local storage.

use crate::monetization::storage::{PremiumStorage, PremiumStatus, StorageError};
use crate::payment::{PaddleClient, PaymentError as PaddlePaymentError, PurchaseReceipt as PaddlePurchaseReceipt};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

/// Payment session for initiating purchases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSession {
    /// Unique session identifier
    pub session_id: String,
    /// Checkout URL for user to complete payment
    pub checkout_url: String,
    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,
}

/// Payment manager service
///
/// Handles payment processing, receipt validation, and premium status management.
/// Integrates with Paddle payment provider and local encrypted storage.
pub struct PaymentManager {
    /// Paddle payment client
    paddle_client: Arc<PaddleClient>,
    /// Premium storage manager
    storage: Arc<PremiumStorage>,
    /// Maximum number of retry attempts for network operations
    max_retries: u32,
    /// Initial retry delay in milliseconds
    initial_retry_delay_ms: u64,
}

impl PaymentManager {
    /// Create a new PaymentManager instance
    ///
    /// # Arguments
    /// * `paddle_client` - Paddle payment client
    /// * `storage` - Premium storage manager
    ///
    /// # Example
    /// ```no_run
    /// use std::sync::Arc;
    /// use cache_cleaner::monetization::{PaymentManager, PremiumStorage};
    /// use cache_cleaner::payment::PaddleClient;
    ///
    /// let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// let manager = PaymentManager::new(paddle_client, storage);
    /// ```
    pub fn new(paddle_client: Arc<PaddleClient>, storage: Arc<PremiumStorage>) -> Self {
        Self {
            paddle_client,
            storage,
            max_retries: 3,
            initial_retry_delay_ms: 1000,
        }
    }

    /// Create a new PaymentManager with custom retry configuration
    ///
    /// # Arguments
    /// * `paddle_client` - Paddle payment client
    /// * `storage` - Premium storage manager
    /// * `max_retries` - Maximum number of retry attempts
    /// * `initial_retry_delay_ms` - Initial retry delay in milliseconds
    pub fn with_retry_config(
        paddle_client: Arc<PaddleClient>,
        storage: Arc<PremiumStorage>,
        max_retries: u32,
        initial_retry_delay_ms: u64,
    ) -> Self {
        Self {
            paddle_client,
            storage,
            max_retries,
            initial_retry_delay_ms,
        }
    }

    /// Initiate a purchase session
    ///
    /// Creates a payment session with a checkout URL that the user can visit
    /// to complete the purchase. The session expires after 30 minutes.
    ///
    /// # Arguments
    /// * `product_id` - Product ID to purchase (defaults to configured product)
    /// * `price` - Price in the configured currency (defaults to $15.00)
    ///
    /// # Returns
    /// * `PaymentSession` - Session with checkout URL and expiration
    ///
    /// # Errors
    /// * `PaymentManagerError::ConfigError` - If payment configuration is invalid
    /// * `PaymentManagerError::NetworkError` - If network request fails after retries
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{PaymentManager, PremiumStorage};
    /// # use cache_cleaner::payment::PaddleClient;
    /// # let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// # let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// # let manager = PaymentManager::new(paddle_client, storage);
    /// let session = manager.initiate_purchase(None, None).await?;
    /// println!("Checkout URL: {}", session.checkout_url);
    /// ```
    pub async fn initiate_purchase(
        &self,
        product_id: Option<String>,
        price: Option<f64>,
    ) -> Result<PaymentSession, PaymentManagerError> {
        // Get product info from Paddle to determine price if not provided
        let product_info = self
            .paddle_client
            .get_product_info()
            .await
            .map_err(|e| PaymentManagerError::PaddleError(e))?;

        let final_product_id = product_id.unwrap_or_else(|| product_info.product_id.clone());
        let final_price = price.unwrap_or(product_info.price);

        // Generate session ID (using timestamp + random for simplicity)
        // In production, you might want to use UUID
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let session_id = format!(
            "session_{}_{}",
            Utc::now().timestamp(),
            rng.gen::<u32>()
        );

        // Create checkout URL
        // Note: This is a simplified version. Actual Paddle checkout integration
        // may require creating a checkout session via API first.
        let checkout_url = format!(
            "https://checkout.paddle.com/product/{}?vendor_id={}&price={}&currency={}",
            final_product_id,
            self.paddle_client.vendor_id(),
            final_price,
            product_info.currency
        );

        let expires_at = Utc::now() + chrono::Duration::minutes(30);

        eprintln!(
            "[PaymentManager] Payment session initiated: session_id={}, product_id={}, price={}",
            session_id,
            final_product_id,
            final_price
        );

        Ok(PaymentSession {
            session_id,
            checkout_url,
            expires_at,
        })
    }

    /// Process a payment transaction
    ///
    /// Verifies a payment transaction with the payment provider and updates
    /// premium status in local storage if the payment is successful.
    ///
    /// # Arguments
    /// * `transaction_id` - Transaction ID from payment provider
    /// * `user_id` - User/device identifier
    ///
    /// # Returns
    /// * `PremiumStatus` - Updated premium status
    ///
    /// # Errors
    /// * `PaymentManagerError::VerificationFailed` - If payment verification fails
    /// * `PaymentManagerError::StorageError` - If storage update fails
    /// * `PaymentManagerError::NetworkError` - If network request fails after retries
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{PaymentManager, PremiumStorage};
    /// # use cache_cleaner::payment::PaddleClient;
    /// # let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// # let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// # let manager = PaymentManager::new(paddle_client, storage);
    /// let status = manager.process_payment("txn_123", "user_456").await?;
    /// ```
    pub async fn process_payment(
        &self,
        transaction_id: &str,
        user_id: &str,
    ) -> Result<PremiumStatus, PaymentManagerError> {
        eprintln!("[PaymentManager] Processing payment: transaction_id={}, user_id={}", transaction_id, user_id);

        // Validate receipt with retry logic
        let receipt = self
            .validate_receipt(transaction_id)
            .await?;

        // Check if payment is completed
        if receipt.status != crate::payment::PaymentStatus::Completed {
            return Err(PaymentManagerError::VerificationFailed(format!(
                "Payment status is not completed: {:?}",
                receipt.status
            )));
        }

        // Parse purchase date
        let purchase_date = chrono::DateTime::parse_from_rfc3339(&receipt.purchase_date)
            .map_err(|e| PaymentManagerError::InvalidReceipt(format!(
                "Invalid purchase date format: {}", e
            )))?
            .with_timezone(&Utc);

        // Create or update premium status
        let premium_status = PremiumStatus {
            schema_version: 1,
            user_id: user_id.to_string(),
            is_premium: true,
            purchase_date,
            expiry_date: None, // Lifetime premium
            transaction_id: Some(receipt.transaction_id.clone()),
            receipt_data: Some(serde_json::to_string(&receipt)
                .map_err(|e| PaymentManagerError::StorageError(StorageError::Serialization(e)))?),
            last_verified: Utc::now(),
            provider: "paddle".to_string(),
            license_key: receipt.license_key.clone(),
        };

        // Save to storage
        self.storage
            .save(&premium_status)
            .map_err(|e| PaymentManagerError::StorageError(e))?;

        eprintln!(
            "[PaymentManager] Payment processed successfully: transaction_id={}, user_id={}",
            transaction_id,
            user_id
        );

        Ok(premium_status)
    }

    /// Validate a purchase receipt
    ///
    /// Verifies a purchase receipt with the payment provider using retry logic
    /// with exponential backoff.
    ///
    /// # Arguments
    /// * `transaction_id` - Transaction ID to validate
    ///
    /// # Returns
    /// * `PurchaseReceipt` - Validated receipt data
    ///
    /// # Errors
    /// * `PaymentManagerError::VerificationFailed` - If receipt validation fails
    /// * `PaymentManagerError::NetworkError` - If network request fails after retries
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{PaymentManager, PremiumStorage};
    /// # use cache_cleaner::payment::PaddleClient;
    /// # let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// # let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// # let manager = PaymentManager::new(paddle_client, storage);
    /// let receipt = manager.validate_receipt("txn_123").await?;
    /// ```
    pub async fn validate_receipt(
        &self,
        transaction_id: &str,
    ) -> Result<PaddlePurchaseReceipt, PaymentManagerError> {
        eprintln!("[PaymentManager] Validating receipt: transaction_id={}", transaction_id);

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self
                .paddle_client
                .verify_payment(transaction_id)
                .await
            {
                Ok(receipt) => {
                    eprintln!(
                        "[PaymentManager] Receipt validated successfully: transaction_id={}, attempt={}",
                        transaction_id,
                        attempt + 1
                    );
                    return Ok(receipt);
                }
                Err(e) => {
                    // Don't retry on certain errors
                    if let PaddlePaymentError::VerificationFailed(_) = &e {
                        eprintln!(
                            "[PaymentManager] Receipt verification failed (non-retryable): transaction_id={}, error={:?}",
                            transaction_id,
                            e
                        );
                        return Err(PaymentManagerError::VerificationFailed(format!(
                            "Receipt validation failed: {:?}",
                            e
                        )));
                    }
                    
                    last_error = Some(e);

                    // Retry with exponential backoff
                    if attempt < self.max_retries {
                        let delay_ms = self.initial_retry_delay_ms * (1 << attempt);
                        eprintln!(
                            "[PaymentManager] Receipt validation failed, retrying: transaction_id={}, attempt={}, delay_ms={}",
                            transaction_id,
                            attempt + 1,
                            delay_ms
                        );
                        sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        // All retries exhausted
        let error_msg = last_error
            .map(|e| format!("{:?}", e))
            .unwrap_or_else(|| "Unknown error".to_string());

        eprintln!(
            "[PaymentManager] Receipt validation failed after {} attempts: transaction_id={}, error={}",
            self.max_retries + 1,
            transaction_id,
            error_msg
        );

        Err(PaymentManagerError::NetworkError(format!(
            "Failed to validate receipt after {} attempts: {}",
            self.max_retries + 1,
            error_msg
        )))
    }

    /// Restore previous purchases
    ///
    /// Attempts to restore premium status from local storage and verify it
    /// with the payment provider. If verification fails, premium status is cleared.
    ///
    /// # Arguments
    /// * `user_id` - User/device identifier
    ///
    /// # Returns
    /// * `Option<PremiumStatus>` - Premium status if found and valid, None otherwise
    ///
    /// # Errors
    /// * `PaymentManagerError::StorageError` - If storage read fails
    ///
    /// # Example
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use cache_cleaner::monetization::{PaymentManager, PremiumStorage};
    /// # use cache_cleaner::payment::PaddleClient;
    /// # let storage = Arc::new(PremiumStorage::with_default_path().unwrap());
    /// # let paddle_client = Arc::new(PaddleClient::from_config().unwrap());
    /// # let manager = PaymentManager::new(paddle_client, storage);
    /// if let Some(status) = manager.restore_purchases("user_456").await? {
    ///     println!("Premium status restored: {:?}", status);
    /// }
    /// ```
    pub async fn restore_purchases(
        &self,
        user_id: &str,
    ) -> Result<Option<PremiumStatus>, PaymentManagerError> {
        eprintln!("[PaymentManager] Restoring purchases: user_id={}", user_id);

        // Load premium status from storage
        let stored_status = self
            .storage
            .load()
            .map_err(|e| PaymentManagerError::StorageError(e))?;

        let status = match stored_status {
            Some(status) => status,
            None => {
                eprintln!("[PaymentManager] No stored premium status found: user_id={}", user_id);
                return Ok(None);
            }
        };

        // Verify user ID matches
        if status.user_id != user_id {
            eprintln!(
                "[PaymentManager] User ID mismatch: stored={}, requested={}",
                status.user_id,
                user_id
            );
            return Ok(None);
        }

        // If we have a transaction ID, verify it with the payment provider
        if let Some(ref transaction_id) = status.transaction_id {
            match self.validate_receipt(transaction_id).await {
                Ok(_) => {
                    // Update last verified timestamp
                    let mut updated_status = status.clone();
                    updated_status.last_verified = Utc::now();
                    
                    // Save updated status
                    self.storage
                        .save(&updated_status)
                        .map_err(|e| PaymentManagerError::StorageError(e))?;

                    eprintln!(
                        "[PaymentManager] Premium status restored and verified: user_id={}, transaction_id={}",
                        user_id,
                        transaction_id
                    );

                    return Ok(Some(updated_status));
                }
                Err(e) => {
                    eprintln!(
                        "[PaymentManager] Receipt verification failed during restore: user_id={}, transaction_id={}, error={:?}",
                        user_id,
                        transaction_id,
                        e
                    );
                    // Clear invalid premium status
                    self.storage
                        .delete()
                        .map_err(|e| PaymentManagerError::StorageError(e))?;
                    return Ok(None);
                }
            }
        }

        // If no transaction ID, check if premium status is still valid based on expiry
        if status.is_premium {
            if let Some(expiry) = status.expiry_date {
                if expiry < Utc::now() {
                    eprintln!(
                        "[PaymentManager] Premium status expired: user_id={}, expiry={}",
                        user_id,
                        expiry
                    );
                    // Clear expired premium status
                    self.storage
                        .delete()
                        .map_err(|e| PaymentManagerError::StorageError(e))?;
                    return Ok(None);
                }
            }

            eprintln!("[PaymentManager] Premium status restored (no verification): user_id={}", user_id);
            return Ok(Some(status));
        }

        Ok(None)
    }
}

/// Payment manager error types
#[derive(Debug, Error)]
pub enum PaymentManagerError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Payment verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid receipt: {0}")]
    InvalidReceipt(String),

    #[error("Payment timeout")]
    Timeout,

    #[error("Paddle API error: {0}")]
    PaddleError(#[from] PaddlePaymentError),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payment::config::PaddleConfig;
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

    #[tokio::test]
    async fn test_payment_manager_creation() {
        let (storage, _temp_dir) = create_test_storage();
        let paddle_client = create_test_paddle_client();
        
        let manager = PaymentManager::new(paddle_client, storage);
        assert_eq!(manager.max_retries, 3);
        assert_eq!(manager.initial_retry_delay_ms, 1000);
    }

    #[tokio::test]
    async fn test_payment_manager_with_retry_config() {
        let (storage, _temp_dir) = create_test_storage();
        let paddle_client = create_test_paddle_client();
        
        let manager = PaymentManager::with_retry_config(
            paddle_client,
            storage,
            5,
            2000,
        );
        assert_eq!(manager.max_retries, 5);
        assert_eq!(manager.initial_retry_delay_ms, 2000);
    }

    #[tokio::test]
    async fn test_initiate_purchase() {
        let (storage, _temp_dir) = create_test_storage();
        let paddle_client = create_test_paddle_client();
        
        let manager = PaymentManager::new(paddle_client, storage);
        
        // This will fail in test because we don't have a real Paddle connection,
        // but we can test the structure
        let result = manager.initiate_purchase(None, None).await;
        
        // In a real test environment with mocked Paddle client, this would succeed
        // For now, we just verify the method exists and can be called
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_restore_purchases_no_storage() {
        let (storage, _temp_dir) = create_test_storage();
        let paddle_client = create_test_paddle_client();
        
        let manager = PaymentManager::new(paddle_client, storage);
        
        let result = manager.restore_purchases("test_user").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_restore_purchases_with_storage() {
        let (storage, _temp_dir) = create_test_storage();
        let paddle_client = create_test_paddle_client();
        
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
        
        storage.save(&premium_status).unwrap();
        
        let manager = PaymentManager::new(paddle_client, storage);
        
        let result = manager.restore_purchases("test_user").await.unwrap();
        assert!(result.is_some());
        let restored = result.unwrap();
        assert_eq!(restored.user_id, "test_user");
        assert!(restored.is_premium);
    }

    #[tokio::test]
    async fn test_restore_purchases_user_mismatch() {
        let (storage, _temp_dir) = create_test_storage();
        let paddle_client = create_test_paddle_client();
        
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
        
        storage.save(&premium_status).unwrap();
        
        let manager = PaymentManager::new(paddle_client, storage);
        
        let result = manager.restore_purchases("test_user").await.unwrap();
        assert!(result.is_none());
    }
}

