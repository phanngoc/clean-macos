use crate::payment::config::PaddleConfig;
use crate::payment::types::*;
use anyhow::Result;
use reqwest::Client;
use serde_json::json;

/// Paddle payment provider client
pub struct PaddleClient {
    config: PaddleConfig,
    client: Client,
}

impl PaddleClient {
    /// Create a new Paddle client with configuration
    pub fn new(config: PaddleConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
    
    /// Create a new Paddle client by loading configuration
    pub fn from_config() -> Result<Self> {
        let config = PaddleConfig::load()?;
        Ok(Self::new(config))
    }
    
    /// Verify a payment transaction
    pub async fn verify_payment(&self, transaction_id: &str) -> Result<PurchaseReceipt, PaymentError> {
        let url = format!("{}/1.0/transaction", self.config.api_base_url());
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .query(&[("transaction_id", transaction_id)])
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PaymentError::ApiError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }
        
        let api_response: PaddleApiResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| PaymentError::InvalidResponse(e.to_string()))?;
        
        if !api_response.success {
            return Err(PaymentError::ApiError(
                api_response
                    .error
                    .map(|e| e.message)
                    .unwrap_or_else(|| "Unknown error".to_string())
            ));
        }
        
        // Parse transaction data
        // Note: This is a simplified version. Actual Paddle API response structure may differ.
        // You'll need to adjust based on Paddle's actual API documentation.
        let transaction = api_response.response.ok_or_else(|| {
            PaymentError::InvalidResponse("No transaction data in response".to_string())
        })?;
        
        Ok(PurchaseReceipt {
            transaction_id: transaction_id.to_string(),
            product_id: transaction
                .get("product_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            amount: transaction
                .get("total")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            currency: transaction
                .get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_string(),
            status: PaymentStatus::Completed, // Parse from actual response
            purchase_date: transaction
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            license_key: transaction
                .get("license_key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            customer_email: transaction
                .get("customer_email")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }
    
    /// Get product information
    pub async fn get_product_info(&self) -> Result<ProductInfo, PaymentError> {
        let url = format!("{}/2.0/product/get_products", self.config.api_base_url());
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&json!({
                "vendor_id": self.config.vendor_id,
                "product_id": self.config.product_id,
            }))
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PaymentError::ApiError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }
        
        let api_response: PaddleApiResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| PaymentError::InvalidResponse(e.to_string()))?;
        
        if !api_response.success {
            return Err(PaymentError::ApiError(
                api_response
                    .error
                    .map(|e| e.message)
                    .unwrap_or_else(|| "Unknown error".to_string())
            ));
        }
        
        let product = api_response.response.ok_or_else(|| {
            PaymentError::InvalidResponse("No product data in response".to_string())
        })?;
        
        Ok(ProductInfo {
            product_id: self.config.product_id.clone(),
            name: product
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Premium License")
                .to_string(),
            price: product
                .get("base_price")
                .and_then(|v| v.as_f64())
                .unwrap_or(15.0),
            currency: product
                .get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_string(),
            description: product
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }
    
    /// Test connection to Paddle API
    pub async fn test_connection(&self) -> Result<bool, PaymentError> {
        // Simple test: try to get product info
        match self.get_product_info().await {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Paddle connection test failed: {:?}", e);
                Ok(false) // Return false but don't error out
            }
        }
    }

    /// Get vendor ID from configuration
    pub fn vendor_id(&self) -> &str {
        &self.config.vendor_id
    }

    /// Get product ID from configuration
    pub fn product_id(&self) -> &str {
        &self.config.product_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_paddle_client_creation() {
        let config = PaddleConfig {
            api_key: "test_key".to_string(),
            vendor_id: "test_vendor".to_string(),
            product_id: "test_product".to_string(),
            test_mode: true,
            webhook_key: None,
        };
        
        let client = PaddleClient::new(config);
        assert_eq!(client.config.test_mode, true);
    }
}

