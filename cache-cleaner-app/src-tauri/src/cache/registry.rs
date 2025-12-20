use super::custom_scanner::{CustomScanner, CustomScannerConfig};
use super::scanner_trait::{CacheCleaner, CacheScanner, CleanResultGeneric, ScanResult};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ScannerRegistry {
    custom_scanners: RwLock<HashMap<String, Arc<CustomScanner>>>,
}

impl ScannerRegistry {
    pub fn new() -> Self {
        Self {
            custom_scanners: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, config: CustomScannerConfig) -> Result<()> {
        let scanner = CustomScanner::new(config.clone())?;
        self.custom_scanners
            .write()
            .await
            .insert(config.id.clone(), Arc::new(scanner));
        Ok(())
    }

    pub async fn unregister(&self, id: &str) -> bool {
        self.custom_scanners.write().await.remove(id).is_some()
    }

    pub async fn get(&self, id: &str) -> Option<Arc<CustomScanner>> {
        self.custom_scanners.read().await.get(id).cloned()
    }

    pub async fn get_all(&self) -> Vec<Arc<CustomScanner>> {
        self.custom_scanners.read().await.values().cloned().collect()
    }

    pub async fn list(&self) -> Vec<CustomScannerConfig> {
        self.custom_scanners
            .read()
            .await
            .values()
            .map(|s| s.config().clone())
            .collect()
    }

    pub async fn scan_all_custom(&self) -> Vec<ScanResult> {
        let scanners: Vec<_> = self.custom_scanners.read().await.values().cloned().collect();
        let mut results = Vec::new();

        for scanner in scanners {
            if let Ok(result) = scanner.scan().await {
                if result.exists && result.size_bytes > 0 {
                    results.push(result);
                }
            }
        }
        results
    }

    pub async fn clean_custom(&self, id: &str, dry_run: bool) -> Result<CleanResultGeneric> {
        let scanner = self
            .get(id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Scanner not found: {}", id))?;
        scanner.clean(dry_run).await
    }
}

impl Default for ScannerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
