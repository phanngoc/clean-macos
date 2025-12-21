use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Paddle payment provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaddleConfig {
    /// Paddle API key (from environment or config)
    pub api_key: String,
    /// Paddle vendor ID
    pub vendor_id: String,
    /// Product ID for $15 premium license
    pub product_id: String,
    /// Test mode flag
    pub test_mode: bool,
    /// Webhook signing key (optional, for webhook validation)
    pub webhook_key: Option<String>,
}

impl PaddleConfig {
    /// Load configuration from environment variables and config file
    /// Priority: Environment variables > Config file > Defaults
    pub fn load() -> Result<Self> {
        // Try to load from environment variables first
        let api_key = std::env::var("PADDLE_API_KEY")
            .or_else(|_| std::env::var("PADDLE_SECRET_KEY"))
            .unwrap_or_else(|_| "test_api_key".to_string());
        
        let vendor_id = std::env::var("PADDLE_VENDOR_ID")
            .unwrap_or_else(|_| "test_vendor_id".to_string());
        
        let product_id = std::env::var("PADDLE_PRODUCT_ID")
            .unwrap_or_else(|_| "test_product_id".to_string());
        
        let test_mode = std::env::var("PADDLE_TEST_MODE")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let webhook_key = std::env::var("PADDLE_WEBHOOK_KEY").ok();
        
        // Try to load from config file (overrides env if exists)
        let config_path = config_path()?;
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(file_config) = serde_json::from_str::<PaddleConfigFile>(&content) {
                    return Ok(Self {
                        api_key: file_config.api_key.unwrap_or(api_key),
                        vendor_id: file_config.vendor_id.unwrap_or(vendor_id),
                        product_id: file_config.product_id.unwrap_or(product_id),
                        test_mode: file_config.test_mode.unwrap_or(test_mode),
                        webhook_key: file_config.webhook_key.or(webhook_key),
                    });
                }
            }
        }
        
        Ok(Self {
            api_key,
            vendor_id,
            product_id,
            test_mode,
            webhook_key,
        })
    }
    
    /// Save configuration to file (for persistence)
    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let file_config = PaddleConfigFile {
            api_key: Some(self.api_key.clone()),
            vendor_id: Some(self.vendor_id.clone()),
            product_id: Some(self.product_id.clone()),
            test_mode: Some(self.test_mode),
            webhook_key: self.webhook_key.clone(),
        };
        
        let content = serde_json::to_string_pretty(&file_config)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
    
    /// Get the base API URL based on test mode
    pub fn api_base_url(&self) -> &'static str {
        if self.test_mode {
            "https://sandbox-api.paddle.com"
        } else {
            "https://api.paddle.com"
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PaddleConfigFile {
    api_key: Option<String>,
    vendor_id: Option<String>,
    product_id: Option<String>,
    test_mode: Option<bool>,
    webhook_key: Option<String>,
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    Ok(home.join(".cache-cleaner/paddle-config.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_load_defaults() {
        // Clear env vars for test
        std::env::remove_var("PADDLE_API_KEY");
        std::env::remove_var("PADDLE_VENDOR_ID");
        std::env::remove_var("PADDLE_PRODUCT_ID");
        std::env::remove_var("PADDLE_TEST_MODE");
        
        let config = PaddleConfig::load().unwrap();
        assert_eq!(config.api_key, "test_api_key");
        assert_eq!(config.vendor_id, "test_vendor_id");
        assert_eq!(config.product_id, "test_product_id");
        assert!(config.test_mode);
    }
    
    #[test]
    fn test_api_base_url() {
        let mut config = PaddleConfig {
            api_key: "test".to_string(),
            vendor_id: "test".to_string(),
            product_id: "test".to_string(),
            test_mode: true,
            webhook_key: None,
        };
        
        assert_eq!(config.api_base_url(), "https://sandbox-api.paddle.com");
        
        config.test_mode = false;
        assert_eq!(config.api_base_url(), "https://api.paddle.com");
    }
}

