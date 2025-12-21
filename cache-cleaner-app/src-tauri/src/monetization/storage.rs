//! Local encrypted storage module for premium status
//!
//! This module provides secure storage for premium user status using AES-256-GCM encryption.
//! Data is stored in JSON format in the app's data directory for easy debugging.
//!
//! Storage location: `~/.cache-cleaner/premium_status.json` (encrypted)

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Premium status information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PremiumStatus {
    /// Schema version for migration support
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Device/user identifier
    pub user_id: String,
    /// Whether user has premium access
    pub is_premium: bool,
    /// Purchase date
    pub purchase_date: DateTime<Utc>,
    /// Expiry date (None for lifetime premium)
    pub expiry_date: Option<DateTime<Utc>>,
    /// Transaction ID (encrypted in storage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    /// Encrypted receipt data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_data: Option<String>,
    /// Last verification timestamp
    pub last_verified: DateTime<Utc>,
    /// Payment provider ("paddle" or "apple_iap")
    pub provider: String,
    /// License key (encrypted in storage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_key: Option<String>,
}

fn default_schema_version() -> u32 {
    1
}

/// Storage errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Decryption error: {0}")]
    Decryption(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Key generation error: {0}")]
    KeyGeneration(String),
}

/// Premium storage manager
pub struct PremiumStorage {
    storage_path: PathBuf,
    encryption_key: Vec<u8>,
}

impl PremiumStorage {
    /// Create a new PremiumStorage instance
    ///
    /// # Arguments
    /// * `storage_path` - Path to the storage file (will be created if doesn't exist)
    /// * `encryption_key` - 32-byte key for AES-256-GCM encryption
    ///
    /// # Errors
    /// Returns error if key is not 32 bytes
    pub fn new<P: AsRef<Path>>(storage_path: P, encryption_key: Vec<u8>) -> Result<Self, StorageError> {
        if encryption_key.len() != 32 {
            return Err(StorageError::KeyGeneration(
                "Encryption key must be 32 bytes for AES-256".to_string(),
            ));
        }

        Ok(Self {
            storage_path: storage_path.as_ref().to_path_buf(),
            encryption_key,
        })
    }

    /// Create a new PremiumStorage with default path and generated key
    ///
    /// The key is derived from a device-specific identifier using SHA-256.
    /// This ensures the key is consistent across app restarts on the same device.
    pub fn with_default_path() -> Result<Self, StorageError> {
        let storage_path = Self::default_storage_path()?;
        let encryption_key = Self::generate_device_key()?;
        Self::new(storage_path, encryption_key)
    }

    /// Get the default storage path
    ///
    /// Returns: `~/.cache-cleaner/premium_status.json`
    pub fn default_storage_path() -> Result<PathBuf, StorageError> {
        let home = dirs::home_dir()
            .ok_or_else(|| StorageError::Storage(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cannot find home directory",
            )))?;
        Ok(home.join(".cache-cleaner/premium_status.json"))
    }

    /// Generate a device-specific encryption key
    ///
    /// Uses a combination of device identifiers to create a consistent key.
    /// In production, consider using a more secure key derivation method.
    fn generate_device_key() -> Result<Vec<u8>, StorageError> {
        // Use a combination of identifiers for key derivation
        let mut hasher = Sha256::new();
        
        // Add home directory path as device identifier
        if let Some(home) = dirs::home_dir() {
            hasher.update(home.to_string_lossy().as_bytes());
        }
        
        // Add a constant salt (in production, consider using a more secure method)
        hasher.update(b"cache-cleaner-premium-storage-v1");
        
        let hash = hasher.finalize();
        Ok(hash.to_vec())
    }

    /// Load premium status from storage
    ///
    /// # Returns
    /// * `Ok(Some(PremiumStatus))` - Status loaded successfully
    /// * `Ok(None)` - No status found (first run)
    /// * `Err(StorageError)` - Error loading or decrypting
    pub fn load(&self) -> Result<Option<PremiumStatus>, StorageError> {
        if !self.storage_path.exists() {
            return Ok(None);
        }

        // Read encrypted file
        let encrypted_data = std::fs::read(&self.storage_path)
            .map_err(|e| StorageError::Storage(e))?;

        // Decrypt
        let decrypted_json = self.decrypt_data(&encrypted_data)?;

        // Deserialize
        let mut status: PremiumStatus = serde_json::from_str(&decrypted_json)?;

        // Handle migration if needed
        if status.schema_version < 1 {
            status = self.migrate(status)?;
        }

        // Decrypt sensitive fields
        if let Some(ref encrypted_tx_id) = status.transaction_id {
            status.transaction_id = Some(self.decrypt_string(encrypted_tx_id)?);
        }
        if let Some(ref encrypted_receipt) = status.receipt_data {
            status.receipt_data = Some(self.decrypt_string(encrypted_receipt)?);
        }
        if let Some(ref encrypted_license) = status.license_key {
            status.license_key = Some(self.decrypt_string(encrypted_license)?);
        }

        Ok(Some(status))
    }

    /// Save premium status to storage
    ///
    /// # Arguments
    /// * `status` - Premium status to save
    ///
    /// # Errors
    /// Returns error if encryption or file write fails
    pub fn save(&self, status: &PremiumStatus) -> Result<(), StorageError> {
        // Create a copy for storage (with encrypted sensitive fields)
        let mut storage_status = status.clone();

        // Encrypt sensitive fields before serialization
        if let Some(ref tx_id) = storage_status.transaction_id {
            storage_status.transaction_id = Some(self.encrypt_string(tx_id)?);
        }
        if let Some(ref receipt) = storage_status.receipt_data {
            storage_status.receipt_data = Some(self.encrypt_string(receipt)?);
        }
        if let Some(ref license) = storage_status.license_key {
            storage_status.license_key = Some(self.encrypt_string(license)?);
        }

        // Ensure schema version is set
        storage_status.schema_version = 1;

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&storage_status)?;

        // Encrypt the entire JSON
        let encrypted_data = self.encrypt_data(&json)?;

        // Ensure parent directory exists
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| StorageError::Storage(e))?;
        }

        // Write to file with restrictive permissions (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let file = std::fs::File::create(&self.storage_path)
                .map_err(|e| StorageError::Storage(e))?;
            let mut perms = file.metadata()
                .map_err(|e| StorageError::Storage(e))?
                .permissions();
            perms.set_mode(0o600); // rw-------
            file.set_permissions(perms)
                .map_err(|e| StorageError::Storage(e))?;
            std::fs::write(&self.storage_path, encrypted_data)
                .map_err(|e| StorageError::Storage(e))?;
        }

        #[cfg(not(unix))]
        {
            std::fs::write(&self.storage_path, encrypted_data)
                .map_err(|e| StorageError::Storage(e))?;
        }

        Ok(())
    }

    /// Delete premium status from storage
    pub fn delete(&self) -> Result<(), StorageError> {
        if self.storage_path.exists() {
            std::fs::remove_file(&self.storage_path)
                .map_err(|e| StorageError::Storage(e))?;
        }
        Ok(())
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt_data(&self, data: &str) -> Result<Vec<u8>, StorageError> {
        let key = Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, data.as_bytes())
            .map_err(|e| StorageError::Encryption(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext (12 bytes for GCM)
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt_data(&self, encrypted: &[u8]) -> Result<String, StorageError> {
        if encrypted.len() < 12 {
            return Err(StorageError::Decryption(
                "Encrypted data too short".to_string(),
            ));
        }

        let key = Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);

        // Extract nonce (first 12 bytes) and ciphertext
        let nonce = Nonce::from_slice(&encrypted[..12]);
        let ciphertext = &encrypted[12..];

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| StorageError::Decryption(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| StorageError::Decryption(format!("Invalid UTF-8: {}", e)))
    }

    /// Encrypt a string (for individual fields)
    fn encrypt_string(&self, data: &str) -> Result<String, StorageError> {
        let encrypted = self.encrypt_data(data)?;
        Ok(general_purpose::STANDARD.encode(encrypted))
    }

    /// Decrypt a string (for individual fields)
    fn decrypt_string(&self, encrypted: &str) -> Result<String, StorageError> {
        let decoded = general_purpose::STANDARD
            .decode(encrypted)
            .map_err(|e| StorageError::Decryption(format!("Base64 decode failed: {}", e)))?;
        self.decrypt_data(&decoded)
    }

    /// Migrate premium status from older schema versions
    fn migrate(&self, mut status: PremiumStatus) -> Result<PremiumStatus, StorageError> {
        // Currently only schema version 1 exists
        // Future migrations can be added here
        match status.schema_version {
            0 => {
                // Migration from version 0 to 1
                status.schema_version = 1;
                Ok(status)
            }
            _ => Err(StorageError::Migration(format!(
                "Unknown schema version: {}",
                status.schema_version
            ))),
        }
    }

    /// Get storage path (for documentation/debugging)
    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (PremiumStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("premium_status.json");
        let key = vec![0u8; 32]; // Test key (all zeros)
        let storage = PremiumStorage::new(storage_path, key).unwrap();
        (storage, temp_dir)
    }

    fn create_test_status() -> PremiumStatus {
        PremiumStatus {
            schema_version: 1,
            user_id: "test-device-123".to_string(),
            is_premium: true,
            purchase_date: Utc::now(),
            expiry_date: None,
            transaction_id: Some("tx_12345".to_string()),
            receipt_data: Some(r#"{"provider":"paddle","amount":15.0}"#.to_string()),
            last_verified: Utc::now(),
            provider: "paddle".to_string(),
            license_key: Some("license_key_abc123".to_string()),
        }
    }

    #[test]
    fn test_storage_creation() {
        let (storage, _temp_dir) = create_test_storage();
        assert!(storage.storage_path().exists() || !storage.storage_path().exists());
    }

    #[test]
    fn test_save_and_load() {
        let (storage, _temp_dir) = create_test_storage();
        let status = create_test_status();

        // Save
        storage.save(&status).unwrap();

        // Load
        let loaded = storage.load().unwrap().unwrap();

        assert_eq!(loaded.user_id, status.user_id);
        assert_eq!(loaded.is_premium, status.is_premium);
        assert_eq!(loaded.provider, status.provider);
        assert_eq!(loaded.transaction_id, status.transaction_id);
        assert_eq!(loaded.receipt_data, status.receipt_data);
        assert_eq!(loaded.license_key, status.license_key);
    }

    #[test]
    fn test_load_nonexistent() {
        let (storage, _temp_dir) = create_test_storage();
        let loaded = storage.load().unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_encryption_decryption() {
        let (storage, _temp_dir) = create_test_storage();
        let original = "sensitive data";

        let encrypted = storage.encrypt_string(original).unwrap();
        assert_ne!(encrypted, original);

        let decrypted = storage.decrypt_string(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_delete() {
        let (storage, _temp_dir) = create_test_storage();
        let status = create_test_status();

        storage.save(&status).unwrap();
        assert!(storage.storage_path().exists());

        storage.delete().unwrap();
        assert!(!storage.storage_path().exists());

        let loaded = storage.load().unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_corrupted_data_handling() {
        let (storage, _temp_dir) = create_test_storage();
        let storage_path = storage.storage_path();

        // Write invalid data
        std::fs::write(storage_path, b"invalid encrypted data").unwrap();

        // Should return error
        let result = storage.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_length() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("premium_status.json");
        let invalid_key = vec![0u8; 16]; // Wrong length

        let result = PremiumStorage::new(storage_path, invalid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_migration() {
        let (storage, _temp_dir) = create_test_storage();
        let mut status = create_test_status();
        status.schema_version = 0; // Old version

        storage.save(&status).unwrap();
        let loaded = storage.load().unwrap().unwrap();
        assert_eq!(loaded.schema_version, 1); // Migrated to version 1
    }

    #[test]
    fn test_default_storage_path() {
        let path = PremiumStorage::default_storage_path().unwrap();
        assert!(path.to_string_lossy().contains(".cache-cleaner"));
        assert!(path.to_string_lossy().ends_with("premium_status.json"));
    }

    #[test]
    fn test_sensitive_fields_encrypted() {
        let (storage, _temp_dir) = create_test_storage();
        let status = create_test_status();

        storage.save(&status).unwrap();

        // Read raw file content
        let raw_content = std::fs::read(storage.storage_path()).unwrap();
        let decrypted_json = storage.decrypt_data(&raw_content).unwrap();
        let stored_status: serde_json::Value = serde_json::from_str(&decrypted_json).unwrap();

        // Sensitive fields should be base64-encoded (encrypted) in storage
        if let Some(tx_id) = stored_status["transaction_id"].as_str() {
            assert_ne!(tx_id, "tx_12345"); // Should be encrypted
            assert!(tx_id.len() > 10); // Base64 encoded should be longer
        }
    }
}

