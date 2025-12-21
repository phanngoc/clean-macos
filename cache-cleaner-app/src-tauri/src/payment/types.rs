use serde::{Deserialize, Serialize};

/// Payment transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
}

/// Purchase receipt from Paddle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseReceipt {
    pub transaction_id: String,
    pub product_id: String,
    pub amount: f64,
    pub currency: String,
    pub status: PaymentStatus,
    pub purchase_date: String,
    pub license_key: Option<String>,
    pub customer_email: Option<String>,
}

/// Payment error types
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("API request failed: {0}")]
    ApiError(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Payment verification failed: {0}")]
    VerificationFailed(String),
}

/// Paddle API response types
#[derive(Debug, Deserialize)]
pub struct PaddleApiResponse<T> {
    pub success: bool,
    pub response: Option<T>,
    pub error: Option<PaddleError>,
}

#[derive(Debug, Deserialize)]
pub struct PaddleError {
    pub message: String,
    pub code: Option<i32>,
}

/// Product information from Paddle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInfo {
    pub product_id: String,
    pub name: String,
    pub price: f64,
    pub currency: String,
    pub description: Option<String>,
}

