//! Centralized error types and error handling for monetization features
//!
//! This module provides a unified error type system for all monetization operations,
//! including ads, payments, and premium status management. It consolidates errors
//! from various services and provides user-friendly error messages.

use crate::monetization::ad_manager::AdError;
use crate::monetization::payment_manager::PaymentManagerError;
use crate::monetization::premium_service::PremiumServiceError;
use crate::monetization::storage::StorageError;
use crate::payment::PaymentError as PaddlePaymentError;
use std::fmt;
use thiserror::Error;

/// Unified error type for all monetization operations
///
/// This enum consolidates errors from all monetization services (ads, payments, premium)
/// and provides a single error type that can be used throughout the monetization layer.
#[derive(Debug, Error)]
pub enum MonetizationError {
    /// Ad-related errors
    #[error("Ad error: {0}")]
    Ad(#[from] AdError),

    /// Payment-related errors
    #[error("Payment error: {0}")]
    Payment(#[from] PaymentManagerError),

    /// Premium service errors
    #[error("Premium service error: {0}")]
    Premium(#[from] PremiumServiceError),

    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Paddle payment provider errors
    #[error("Paddle payment error: {0}")]
    Paddle(#[from] PaddlePaymentError),

    /// Network/connectivity errors
    #[error("Network error: {0}")]
    Network(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Invalid input/parameter errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Unknown/unexpected errors
    #[error("Unexpected error: {0}")]
    Unknown(String),
}

/// Error severity level for logging and user notification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Low severity - informational, can be ignored
    Low,
    /// Medium severity - warning, user should be aware
    Medium,
    /// High severity - error, operation failed
    High,
    /// Critical severity - system error, requires attention
    Critical,
}

impl MonetizationError {
    /// Get user-friendly error message
    ///
    /// Returns a message suitable for display to end users, avoiding
    /// technical details that might confuse users.
    pub fn user_message(&self) -> String {
        match self {
            MonetizationError::Ad(AdError::NotRequired) => {
                "You have premium access. Ads are not required.".to_string()
            }
            MonetizationError::Ad(AdError::LoadFailed(msg)) => {
                format!("Unable to load advertisement. Please try again. ({})", msg)
            }
            MonetizationError::Ad(AdError::Timeout) => {
                "Advertisement timed out. Please try again.".to_string()
            }
            MonetizationError::Ad(AdError::Blocked) => {
                "Advertisement was blocked. Please disable ad blockers.".to_string()
            }
            MonetizationError::Ad(AdError::SdkError(msg)) => {
                format!("Advertisement service error. Please try again later. ({})", msg)
            }
            MonetizationError::Ad(AdError::AdNotFound(id)) => {
                format!("Advertisement not found. Please request a new ad. (ID: {})", id)
            }
            MonetizationError::Ad(AdError::InvalidConfig(msg)) => {
                format!("Advertisement configuration error. Please contact support. ({})", msg)
            }

            MonetizationError::Payment(PaymentManagerError::NetworkError(msg)) => {
                format!("Network error while processing payment. Please check your connection. ({})", msg)
            }
            MonetizationError::Payment(PaymentManagerError::VerificationFailed(msg)) => {
                format!("Payment verification failed. Please contact support. ({})", msg)
            }
            MonetizationError::Payment(PaymentManagerError::InvalidReceipt(msg)) => {
                format!("Invalid payment receipt. Please contact support. ({})", msg)
            }
            MonetizationError::Payment(PaymentManagerError::Timeout) => {
                "Payment processing timed out. Please try again.".to_string()
            }
            MonetizationError::Payment(PaymentManagerError::PaddleError(e)) => {
                format!("Payment provider error. Please try again later. ({})", e)
            }
            MonetizationError::Payment(PaymentManagerError::ConfigError(msg)) => {
                format!("Payment configuration error. Please contact support. ({})", msg)
            }
            MonetizationError::Payment(PaymentManagerError::StorageError(e)) => {
                format!("Storage error while processing payment. Please try again. ({})", e)
            }

            MonetizationError::Premium(PremiumServiceError::VerificationFailed(msg)) => {
                format!("Premium status verification failed. Please try again. ({})", msg)
            }
            MonetizationError::Premium(PremiumServiceError::InvalidStatus(msg)) => {
                format!("Invalid premium status. Please contact support. ({})", msg)
            }
            MonetizationError::Premium(PremiumServiceError::DeviceIdError(msg)) => {
                format!("Device identification error. Please restart the app. ({})", msg)
            }
            MonetizationError::Premium(e) => {
                format!("Premium service error. Please try again. ({})", e)
            }

            MonetizationError::Storage(StorageError::Storage(e)) => {
                format!("Storage error. Please check file permissions. ({})", e)
            }
            MonetizationError::Storage(StorageError::Encryption(msg)) => {
                format!("Encryption error. Please restart the app. ({})", msg)
            }
            MonetizationError::Storage(StorageError::Decryption(msg)) => {
                format!("Decryption error. Please restart the app. ({})", msg)
            }
            MonetizationError::Storage(StorageError::Serialization(e)) => {
                format!("Data format error. Please restart the app. ({})", e)
            }
            MonetizationError::Storage(StorageError::InvalidData(msg)) => {
                format!("Invalid data. Please restart the app. ({})", msg)
            }
            MonetizationError::Storage(e) => {
                format!("Storage error. Please restart the app. ({})", e)
            }

            MonetizationError::Paddle(e) => {
                format!("Payment provider error. Please try again later. ({})", e)
            }

            MonetizationError::Network(msg) => {
                format!("Network error. Please check your internet connection. ({})", msg)
            }
            MonetizationError::Config(msg) => {
                format!("Configuration error. Please contact support. ({})", msg)
            }
            MonetizationError::Timeout(msg) => {
                format!("Operation timed out. Please try again. ({})", msg)
            }
            MonetizationError::InvalidInput(msg) => {
                format!("Invalid input. Please check your request. ({})", msg)
            }
            MonetizationError::Unknown(msg) => {
                format!("An unexpected error occurred. Please try again. ({})", msg)
            }
        }
    }

    /// Get error severity level
    ///
    /// Returns the severity level for logging and user notification purposes.
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Low severity - informational
            MonetizationError::Ad(AdError::NotRequired) => ErrorSeverity::Low,

            // Medium severity - warnings
            MonetizationError::Ad(AdError::Timeout) => ErrorSeverity::Medium,
            MonetizationError::Ad(AdError::Blocked) => ErrorSeverity::Medium,
            MonetizationError::Payment(PaymentManagerError::Timeout) => ErrorSeverity::Medium,
            MonetizationError::Network(_) => ErrorSeverity::Medium,

            // High severity - errors
            MonetizationError::Ad(_) => ErrorSeverity::High,
            MonetizationError::Payment(_) => ErrorSeverity::High,
            MonetizationError::Premium(_) => ErrorSeverity::High,
            MonetizationError::Paddle(_) => ErrorSeverity::High,
            MonetizationError::Config(_) => ErrorSeverity::High,
            MonetizationError::InvalidInput(_) => ErrorSeverity::High,
            MonetizationError::Timeout(_) => ErrorSeverity::High,

            // Critical severity - system errors
            MonetizationError::Storage(_) => ErrorSeverity::Critical,
            MonetizationError::Unknown(_) => ErrorSeverity::Critical,
        }
    }

    /// Check if error is retryable
    ///
    /// Returns true if the operation that caused this error can be safely retried.
    pub fn is_retryable(&self) -> bool {
        match self {
            MonetizationError::Ad(AdError::LoadFailed(_)) => true,
            MonetizationError::Ad(AdError::Timeout) => true,
            MonetizationError::Ad(AdError::SdkError(_)) => true,
            MonetizationError::Payment(PaymentManagerError::NetworkError(_)) => true,
            MonetizationError::Payment(PaymentManagerError::Timeout) => true,
            MonetizationError::Payment(PaymentManagerError::PaddleError(_)) => true,
            MonetizationError::Premium(PremiumServiceError::VerificationFailed(_)) => true,
            MonetizationError::Network(_) => true,
            MonetizationError::Timeout(_) => true,
            _ => false,
        }
    }

    /// Log error with appropriate level
    ///
    /// Logs the error using eprintln! with severity-appropriate formatting.
    pub fn log(&self, context: &str) {
        let severity = self.severity();
        let message = self.user_message();
        
        match severity {
            ErrorSeverity::Low => {
                eprintln!("[Monetization] [INFO] [{}] {}", context, message);
            }
            ErrorSeverity::Medium => {
                eprintln!("[Monetization] [WARN] [{}] {}", context, message);
            }
            ErrorSeverity::High => {
                eprintln!("[Monetization] [ERROR] [{}] {}", context, message);
            }
            ErrorSeverity::Critical => {
                eprintln!("[Monetization] [CRITICAL] [{}] {}", context, message);
            }
        }
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "Low"),
            ErrorSeverity::Medium => write!(f, "Medium"),
            ErrorSeverity::High => write!(f, "High"),
            ErrorSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Result type alias for monetization operations
pub type MonetizationResult<T> = Result<T, MonetizationError>;

/// Helper trait for converting errors to MonetizationError
pub trait ToMonetizationError<T> {
    /// Convert to MonetizationError with context
    fn to_monetization_error(self, context: &str) -> Result<T, MonetizationError>;
}

impl<T, E: Into<MonetizationError>> ToMonetizationError<T> for Result<T, E> {
    fn to_monetization_error(self, _context: &str) -> Result<T, MonetizationError> {
        self.map_err(|e| e.into())
    }
}

/// Retry configuration for operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Maximum delay in milliseconds (for exponential backoff)
    pub max_delay_ms: Option<u64>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            exponential_backoff: true,
            max_delay_ms: Some(10000), // 10 seconds max
        }
    }
}

/// Retry an operation with exponential backoff
///
/// This function will retry the operation up to `max_retries` times if the error
/// is retryable. It uses exponential backoff between retries.
///
/// # Arguments
/// * `operation` - The async operation to retry
/// * `config` - Retry configuration
/// * `context` - Context string for logging
///
/// # Returns
/// * `Ok(T)` - Operation succeeded
/// * `Err(MonetizationError)` - Operation failed after all retries
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    config: RetryConfig,
    context: &str,
) -> Result<T, MonetizationError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, MonetizationError>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    eprintln!(
                        "[Monetization] [RETRY] [{}] Operation succeeded after {} attempts",
                        context, attempt + 1
                    );
                }
                return Ok(result);
            }
                Err(e) => {
                // Check if error is retryable
                if !e.is_retryable() {
                    e.log(context);
                    return Err(e);
                }

                // Store error message for final error if all retries fail
                let error_msg = e.user_message();
                last_error = Some(error_msg);

                // Retry with exponential backoff
                if attempt < config.max_retries {
                    let delay_ms = if config.exponential_backoff {
                        let calculated = config.initial_delay_ms * (1 << attempt);
                        config.max_delay_ms
                            .map(|max| calculated.min(max))
                            .unwrap_or(calculated)
                    } else {
                        config.initial_delay_ms
                    };

                    eprintln!(
                        "[Monetization] [RETRY] [{}] Attempt {} failed, retrying in {}ms: {}",
                        context,
                        attempt + 1,
                        delay_ms,
                        e.user_message()
                    );

                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }

    // All retries exhausted
    let error_msg = last_error.unwrap_or_else(|| {
        format!("Operation failed after {} attempts", config.max_retries + 1)
    });
    
    let error = MonetizationError::Unknown(error_msg.clone());

    eprintln!(
        "[Monetization] [RETRY] [{}] Operation failed after {} attempts: {}",
        context,
        config.max_retries + 1,
        error_msg
    );

    error.log(context);
    Err(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message() {
        let error = MonetizationError::Ad(AdError::NotRequired);
        let msg = error.user_message();
        assert!(msg.contains("premium"));
        assert!(msg.contains("not required"));
    }

    #[test]
    fn test_severity() {
        let error = MonetizationError::Ad(AdError::NotRequired);
        assert_eq!(error.severity(), ErrorSeverity::Low);

        let error = MonetizationError::Storage(StorageError::Encryption("test".to_string()));
        assert_eq!(error.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_retryable() {
        let error = MonetizationError::Ad(AdError::LoadFailed("test".to_string()));
        assert!(error.is_retryable());

        let error = MonetizationError::Ad(AdError::NotRequired);
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert!(config.exponential_backoff);
    }
}

