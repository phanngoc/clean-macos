// Monetization module for ads and payment integration
pub mod storage;
pub mod payment_manager;
pub mod premium_service;
pub mod ad_manager;
pub mod errors;

pub use storage::{PremiumStatus, StorageError, PremiumStorage};
pub use payment_manager::{PaymentManager, PaymentManagerError, PaymentSession};
pub use premium_service::{PremiumService, PremiumServiceError};
pub use ad_manager::{AdManager, AdConfig, AdRequest, AdError, AdEvent, AdEventType};
pub use errors::{
    MonetizationError, MonetizationResult, ErrorSeverity, ToMonetizationError,
    RetryConfig, retry_with_backoff,
};

