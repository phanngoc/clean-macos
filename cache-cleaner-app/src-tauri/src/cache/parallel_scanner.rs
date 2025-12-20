use super::custom_scanner::CustomScanner;
use super::registry::ScannerRegistry;
use super::scanner_trait::{CacheScanner, ScanResult};
use crate::utils::concurrency::{create_semaphore, DEFAULT_CONCURRENCY};
use std::sync::Arc;

pub async fn scan_all_parallel(registry: &ScannerRegistry) -> Vec<ScanResult> {
    let scanners: Vec<Arc<CustomScanner>> = registry.get_all().await;
    if scanners.is_empty() {
        return Vec::new();
    }

    let semaphore = create_semaphore(DEFAULT_CONCURRENCY);
    let mut handles = Vec::new();

    for scanner in scanners {
        let sem = semaphore.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.ok()?;
            scanner.scan().await.ok()
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(Some(result)) = handle.await {
            if result.exists && result.size_bytes > 0 {
                results.push(result);
            }
        }
    }
    results
}
