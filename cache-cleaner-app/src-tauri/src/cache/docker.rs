//! Docker cleanup module for scanning and removing obsolete Docker resources.
//! 
//! This module provides functionality to:
//! - Scan Docker containers, images, volumes, and networks
//! - Identify dangling/unused resources
//! - Clean up resources with smart suggestions
//! - Handle Docker daemon connectivity

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::collections::HashSet;

// ============================================================================
// Data Structures
// ============================================================================

/// Container state enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContainerState {
    Running,
    Exited,
    Created,
    Paused,
    Restarting,
    Dead,
    Removing,
    Unknown,
}

impl From<&str> for ContainerState {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" => ContainerState::Running,
            "exited" => ContainerState::Exited,
            "created" => ContainerState::Created,
            "paused" => ContainerState::Paused,
            "restarting" => ContainerState::Restarting,
            "dead" => ContainerState::Dead,
            "removing" => ContainerState::Removing,
            _ => ContainerState::Unknown,
        }
    }
}

/// Docker resource type for suggestions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DockerResourceType {
    Container,
    Image,
    Volume,
    Network,
    BuildCache,
}

/// Represents a Docker container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: ContainerState,
    pub size: u64,
    pub created: String,
    pub ports: String,
}

/// Represents a Docker image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: u64,
    pub created: String,
    pub is_dangling: bool,
    pub used_by_containers: Vec<String>,
}

/// Represents a Docker volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerVolume {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub size: Option<u64>,
    pub used_by_containers: Vec<String>,
}

/// Represents a Docker network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerNetwork {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub used_by_containers: Vec<String>,
}

/// Result of scanning all Docker resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerScanResult {
    pub daemon_running: bool,
    pub containers: Vec<DockerContainer>,
    pub images: Vec<DockerImage>,
    pub volumes: Vec<DockerVolume>,
    pub networks: Vec<DockerNetwork>,
    pub build_cache_size: u64,
    pub total_reclaimable: u64,
    pub stopped_containers_count: usize,
    pub dangling_images_count: usize,
    pub unused_images_count: usize,
    pub orphan_volumes_count: usize,
    pub unused_networks_count: usize,
}

/// Result of a Docker cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerCleanResult {
    pub freed_bytes: u64,
    pub containers_removed: usize,
    pub images_removed: usize,
    pub volumes_removed: usize,
    pub networks_removed: usize,
    pub success: bool,
    pub message: String,
}

/// Smart suggestion for a Docker resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerSuggestion {
    pub resource_type: DockerResourceType,
    pub id: String,
    pub name: String,
    pub size: u64,
    pub score: f64,
    pub reasons: Vec<String>,
    pub auto_select: bool,
}

/// Docker error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockerError {
    DaemonNotRunning,
    DockerNotInstalled,
    PermissionDenied,
    ContainerInUse,
    ImageInUse,
    NetworkInUse,
    CommandFailed(String),
}

// Default networks that should not be removed
const DEFAULT_NETWORKS: &[&str] = &["bridge", "host", "none"];

// Smart suggestion scoring weights
const SIZE_WEIGHT: f64 = 0.3;
const AGE_WEIGHT: f64 = 0.3;
const USAGE_WEIGHT: f64 = 0.4;

// Size thresholds for scoring (in bytes)
const SIZE_LARGE: u64 = 1024 * 1024 * 1024; // 1GB
const SIZE_MEDIUM: u64 = 500 * 1024 * 1024; // 500MB
const SIZE_SMALL: u64 = 100 * 1024 * 1024;  // 100MB

// ============================================================================
// Docker Daemon Check
// ============================================================================

/// Check if Docker is installed
pub fn is_docker_installed() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Docker daemon is running
pub async fn is_docker_running() -> bool {
    tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["info"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

// ============================================================================
// Scanning Functions
// ============================================================================

/// Scan all Docker resources
pub async fn scan_docker_resources() -> Result<DockerScanResult> {
    if !is_docker_running().await {
        return Ok(DockerScanResult {
            daemon_running: false,
            containers: vec![],
            images: vec![],
            volumes: vec![],
            networks: vec![],
            build_cache_size: 0,
            total_reclaimable: 0,
            stopped_containers_count: 0,
            dangling_images_count: 0,
            unused_images_count: 0,
            orphan_volumes_count: 0,
            unused_networks_count: 0,
        });
    }

    // Scan all resources in parallel
    let (containers, images, volumes, networks, build_cache_size) = tokio::join!(
        scan_containers(),
        scan_images(),
        scan_volumes(),
        scan_networks(),
        get_build_cache_size(),
    );

    let containers = containers.unwrap_or_default();
    let images = images.unwrap_or_default();
    let volumes = volumes.unwrap_or_default();
    let networks = networks.unwrap_or_default();
    let build_cache_size = build_cache_size.unwrap_or(0);

    // Calculate counts
    let stopped_containers_count = containers
        .iter()
        .filter(|c| c.state != ContainerState::Running)
        .count();
    
    let dangling_images_count = images.iter().filter(|i| i.is_dangling).count();
    let unused_images_count = images
        .iter()
        .filter(|i| i.used_by_containers.is_empty())
        .count();
    
    let orphan_volumes_count = volumes
        .iter()
        .filter(|v| v.used_by_containers.is_empty())
        .count();
    
    let unused_networks_count = networks
        .iter()
        .filter(|n| {
            n.used_by_containers.is_empty() && !DEFAULT_NETWORKS.contains(&n.name.as_str())
        })
        .count();

    // Calculate total reclaimable space
    let container_size: u64 = containers
        .iter()
        .filter(|c| c.state != ContainerState::Running)
        .map(|c| c.size)
        .sum();
    
    let unused_image_size: u64 = images
        .iter()
        .filter(|i| i.used_by_containers.is_empty())
        .map(|i| i.size)
        .sum();
    
    let orphan_volume_size: u64 = volumes
        .iter()
        .filter(|v| v.used_by_containers.is_empty())
        .filter_map(|v| v.size)
        .sum();

    let total_reclaimable = container_size + unused_image_size + orphan_volume_size + build_cache_size;

    Ok(DockerScanResult {
        daemon_running: true,
        containers,
        images,
        volumes,
        networks,
        build_cache_size,
        total_reclaimable,
        stopped_containers_count,
        dangling_images_count,
        unused_images_count,
        orphan_volumes_count,
        unused_networks_count,
    })
}

/// Scan all containers
async fn scan_containers() -> Result<Vec<DockerContainer>> {
    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args([
                "ps", "-a", "--no-trunc",
                "--format", "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.State}}\t{{.Size}}\t{{.CreatedAt}}\t{{.Ports}}"
            ])
            .output()
    })
    .await??;

    if !output.status.success() {
        return Err(anyhow!("Failed to list containers"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 6 {
            let size = parse_docker_size(parts.get(5).unwrap_or(&"0"));
            
            containers.push(DockerContainer {
                id: parts[0].to_string(),
                name: parts[1].to_string(),
                image: parts[2].to_string(),
                status: parts[3].to_string(),
                state: ContainerState::from(*parts.get(4).unwrap_or(&"unknown")),
                size,
                created: parts.get(6).unwrap_or(&"").to_string(),
                ports: parts.get(7).unwrap_or(&"").to_string(),
            });
        }
    }

    Ok(containers)
}

/// Scan all images
async fn scan_images() -> Result<Vec<DockerImage>> {
    // Get all images
    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args([
                "images", "-a", "--no-trunc",
                "--format", "{{.ID}}\t{{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
            ])
            .output()
    })
    .await??;

    if !output.status.success() {
        return Err(anyhow!("Failed to list images"));
    }

    // Get dangling image IDs
    let dangling_output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["images", "-f", "dangling=true", "-q", "--no-trunc"])
            .output()
    })
    .await??;

    let dangling_ids: HashSet<String> = String::from_utf8_lossy(&dangling_output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Get container image usage
    let container_images = get_container_image_usage().await.unwrap_or_default();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut images = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            let id = parts[0].to_string();
            let repository = parts[1].to_string();
            let tag = parts[2].to_string();
            let size = parse_docker_size(parts[3]);
            
            let is_dangling = dangling_ids.contains(&id) 
                || (repository == "<none>" && tag == "<none>");
            
            let used_by = container_images
                .get(&id)
                .cloned()
                .unwrap_or_default();

            images.push(DockerImage {
                id,
                repository,
                tag,
                size,
                created: parts.get(4).unwrap_or(&"").to_string(),
                is_dangling,
                used_by_containers: used_by,
            });
        }
    }

    Ok(images)
}

/// Get which containers are using which images
async fn get_container_image_usage() -> Result<std::collections::HashMap<String, Vec<String>>> {
    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["ps", "-a", "--no-trunc", "--format", "{{.ID}}\t{{.Image}}"])
            .output()
    })
    .await??;

    let mut usage: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    // Also get image IDs for each container
    let image_id_output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["inspect", "--format", "{{.Id}}\t{{.Image}}", "-a"])
            .output()
    })
    .await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let container_id = parts[0].to_string();
            let image_ref = parts[1].to_string();
            
            // Try to resolve image name to ID
            if let Ok(Ok(inspect_output)) = &image_id_output {
                let inspect_str = String::from_utf8_lossy(&inspect_output.stdout);
                for inspect_line in inspect_str.lines() {
                    let inspect_parts: Vec<&str> = inspect_line.split('\t').collect();
                    if inspect_parts.len() >= 2 {
                        let image_id = inspect_parts[1].trim_start_matches("sha256:");
                        usage.entry(image_id.to_string())
                            .or_default()
                            .push(container_id.clone());
                    }
                }
            }
            
            // Also add by image name/tag
            usage.entry(image_ref.clone())
                .or_default()
                .push(container_id);
        }
    }

    Ok(usage)
}

/// Scan all volumes
async fn scan_volumes() -> Result<Vec<DockerVolume>> {
    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["volume", "ls", "--format", "{{.Name}}\t{{.Driver}}\t{{.Mountpoint}}"])
            .output()
    })
    .await??;

    if !output.status.success() {
        return Err(anyhow!("Failed to list volumes"));
    }

    // Get volume usage from containers
    let volume_usage = get_volume_usage().await.unwrap_or_default();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut volumes = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let name = parts[0].to_string();
            let used_by = volume_usage.get(&name).cloned().unwrap_or_default();
            
            // Try to get volume size
            let size = get_volume_size(&name).await.ok();

            volumes.push(DockerVolume {
                name: name.clone(),
                driver: parts[1].to_string(),
                mountpoint: parts.get(2).unwrap_or(&"").to_string(),
                size,
                used_by_containers: used_by,
            });
        }
    }

    Ok(volumes)
}

/// Get volume usage by containers
async fn get_volume_usage() -> Result<std::collections::HashMap<String, Vec<String>>> {
    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["ps", "-a", "--format", "{{.ID}}\t{{.Mounts}}"])
            .output()
    })
    .await??;

    let mut usage: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let container_id = parts[0].to_string();
            let mounts = parts[1];
            
            for mount in mounts.split(',') {
                let mount_name = mount.trim();
                if !mount_name.is_empty() {
                    usage.entry(mount_name.to_string())
                        .or_default()
                        .push(container_id.clone());
                }
            }
        }
    }

    Ok(usage)
}

/// Get volume size (approximate)
async fn get_volume_size(name: &str) -> Result<u64> {
    let name = name.to_string();
    let output = tokio::task::spawn_blocking(move || {
        Command::new("docker")
            .args(["system", "df", "-v", "--format", "{{json .}}"])
            .output()
    })
    .await??;

    // Parse JSON output to find volume size
    // This is simplified - in production, would parse the full JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // For now, return 0 as getting exact volume size requires more complex parsing
    // The docker system df command gives aggregate info, not per-volume
    if stdout.contains(&name) {
        Ok(0) // Placeholder - would need proper JSON parsing
    } else {
        Ok(0)
    }
}

/// Scan all networks
async fn scan_networks() -> Result<Vec<DockerNetwork>> {
    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["network", "ls", "--no-trunc", "--format", "{{.ID}}\t{{.Name}}\t{{.Driver}}\t{{.Scope}}"])
            .output()
    })
    .await??;

    if !output.status.success() {
        return Err(anyhow!("Failed to list networks"));
    }

    // Get network usage
    let network_usage = get_network_usage().await.unwrap_or_default();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let name = parts[1].to_string();
            let used_by = network_usage.get(&name).cloned().unwrap_or_default();

            networks.push(DockerNetwork {
                id: parts[0].to_string(),
                name,
                driver: parts[2].to_string(),
                scope: parts.get(3).unwrap_or(&"").to_string(),
                used_by_containers: used_by,
            });
        }
    }

    Ok(networks)
}

/// Get network usage by containers
async fn get_network_usage() -> Result<std::collections::HashMap<String, Vec<String>>> {
    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["ps", "-a", "--format", "{{.ID}}\t{{.Networks}}"])
            .output()
    })
    .await??;

    let mut usage: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let container_id = parts[0].to_string();
            let networks = parts[1];
            
            for network in networks.split(',') {
                let network_name = network.trim();
                if !network_name.is_empty() {
                    usage.entry(network_name.to_string())
                        .or_default()
                        .push(container_id.clone());
                }
            }
        }
    }

    Ok(usage)
}

/// Get build cache size
async fn get_build_cache_size() -> Result<u64> {
    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["system", "df", "--format", "{{.Type}}\t{{.Size}}"])
            .output()
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 && parts[0].to_lowercase().contains("build") {
            return Ok(parse_docker_size(parts[1]));
        }
    }

    Ok(0)
}

// ============================================================================
// Filtering Functions
// ============================================================================

/// Get stopped containers
pub async fn get_stopped_containers() -> Result<Vec<DockerContainer>> {
    let result = scan_docker_resources().await?;
    Ok(result
        .containers
        .into_iter()
        .filter(|c| c.state != ContainerState::Running)
        .collect())
}

/// Get dangling images
pub async fn get_dangling_images() -> Result<Vec<DockerImage>> {
    let result = scan_docker_resources().await?;
    Ok(result.images.into_iter().filter(|i| i.is_dangling).collect())
}

/// Get unused images (not used by any container)
pub async fn get_unused_images() -> Result<Vec<DockerImage>> {
    let result = scan_docker_resources().await?;
    Ok(result
        .images
        .into_iter()
        .filter(|i| i.used_by_containers.is_empty())
        .collect())
}

/// Get orphan volumes (not used by any container)
pub async fn get_orphan_volumes() -> Result<Vec<DockerVolume>> {
    let result = scan_docker_resources().await?;
    Ok(result
        .volumes
        .into_iter()
        .filter(|v| v.used_by_containers.is_empty())
        .collect())
}

/// Get unused networks (not default and not used by any container)
pub async fn get_unused_networks() -> Result<Vec<DockerNetwork>> {
    let result = scan_docker_resources().await?;
    Ok(result
        .networks
        .into_iter()
        .filter(|n| {
            n.used_by_containers.is_empty() && !DEFAULT_NETWORKS.contains(&n.name.as_str())
        })
        .collect())
}

// ============================================================================
// Cleanup Functions
// ============================================================================

/// Remove specific containers
pub async fn remove_containers(ids: Vec<String>, force: bool) -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    if ids.is_empty() {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: true,
            message: "No containers to remove".to_string(),
        });
    }

    let mut removed = 0;
    let mut freed_bytes = 0u64;
    let mut errors = Vec::new();

    // Get sizes before removal
    let containers = scan_containers().await.unwrap_or_default();
    let size_map: std::collections::HashMap<String, u64> = containers
        .into_iter()
        .map(|c| (c.id.clone(), c.size))
        .collect();

    for id in &ids {
        let size = size_map.get(id).copied().unwrap_or(0);
        
        let mut args = vec!["rm".to_string()];
        if force {
            args.push("-f".to_string());
        }
        args.push(id.clone());

        let id_clone = id.clone();
        let args_clone = args.clone();
        
        let output = tokio::task::spawn_blocking(move || {
            Command::new("docker").args(&args_clone).output()
        })
        .await??;

        if output.status.success() {
            removed += 1;
            freed_bytes += size;
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            errors.push(format!("{}: {}", id_clone, stderr.trim()));
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Successfully removed {} container(s)", removed)
    } else {
        format!(
            "Removed {} container(s) with {} error(s): {}",
            removed,
            errors.len(),
            errors.join("; ")
        )
    };

    Ok(DockerCleanResult {
        freed_bytes,
        containers_removed: removed,
        images_removed: 0,
        volumes_removed: 0,
        networks_removed: 0,
        success,
        message,
    })
}

/// Remove specific images
pub async fn remove_images(ids: Vec<String>, force: bool) -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    if ids.is_empty() {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: true,
            message: "No images to remove".to_string(),
        });
    }

    let mut removed = 0;
    let mut freed_bytes = 0u64;
    let mut errors = Vec::new();

    // Get sizes before removal
    let images = scan_images().await.unwrap_or_default();
    let size_map: std::collections::HashMap<String, u64> = images
        .into_iter()
        .map(|i| (i.id.clone(), i.size))
        .collect();

    for id in &ids {
        let size = size_map.get(id).copied().unwrap_or(0);
        
        let mut args = vec!["rmi".to_string()];
        if force {
            args.push("-f".to_string());
        }
        args.push(id.clone());

        let id_clone = id.clone();
        let args_clone = args.clone();
        
        let output = tokio::task::spawn_blocking(move || {
            Command::new("docker").args(&args_clone).output()
        })
        .await??;

        if output.status.success() {
            removed += 1;
            freed_bytes += size;
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            errors.push(format!("{}: {}", id_clone, stderr.trim()));
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Successfully removed {} image(s)", removed)
    } else {
        format!(
            "Removed {} image(s) with {} error(s): {}",
            removed,
            errors.len(),
            errors.join("; ")
        )
    };

    Ok(DockerCleanResult {
        freed_bytes,
        containers_removed: 0,
        images_removed: removed,
        volumes_removed: 0,
        networks_removed: 0,
        success,
        message,
    })
}

/// Remove specific volumes
pub async fn remove_volumes(names: Vec<String>) -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    if names.is_empty() {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: true,
            message: "No volumes to remove".to_string(),
        });
    }

    let mut removed = 0;
    let mut errors = Vec::new();

    for name in &names {
        let name_clone = name.clone();
        
        let output = tokio::task::spawn_blocking(move || {
            Command::new("docker")
                .args(["volume", "rm", &name_clone])
                .output()
        })
        .await??;

        if output.status.success() {
            removed += 1;
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            errors.push(format!("{}: {}", name, stderr.trim()));
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Successfully removed {} volume(s)", removed)
    } else {
        format!(
            "Removed {} volume(s) with {} error(s): {}",
            removed,
            errors.len(),
            errors.join("; ")
        )
    };

    Ok(DockerCleanResult {
        freed_bytes: 0, // Volume sizes are hard to determine
        containers_removed: 0,
        images_removed: 0,
        volumes_removed: removed,
        networks_removed: 0,
        success,
        message,
    })
}

/// Remove specific networks
pub async fn remove_networks(ids: Vec<String>) -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    // Filter out default networks
    let ids: Vec<String> = ids
        .into_iter()
        .filter(|id| !DEFAULT_NETWORKS.contains(&id.as_str()))
        .collect();

    if ids.is_empty() {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: true,
            message: "No networks to remove".to_string(),
        });
    }

    let mut removed = 0;
    let mut errors = Vec::new();

    for id in &ids {
        let id_clone = id.clone();
        
        let output = tokio::task::spawn_blocking(move || {
            Command::new("docker")
                .args(["network", "rm", &id_clone])
                .output()
        })
        .await??;

        if output.status.success() {
            removed += 1;
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            errors.push(format!("{}: {}", id, stderr.trim()));
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Successfully removed {} network(s)", removed)
    } else {
        format!(
            "Removed {} network(s) with {} error(s): {}",
            removed,
            errors.len(),
            errors.join("; ")
        )
    };

    Ok(DockerCleanResult {
        freed_bytes: 0,
        containers_removed: 0,
        images_removed: 0,
        volumes_removed: 0,
        networks_removed: removed,
        success,
        message,
    })
}

/// Prune all unused Docker resources
pub async fn docker_system_prune(all: bool, volumes: bool) -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    let mut args = vec!["system", "prune", "-f"];
    if all {
        args.push("-a");
    }
    if volumes {
        args.push("--volumes");
    }

    let output = tokio::task::spawn_blocking(move || {
        Command::new("docker").args(&args).output()
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse reclaimed space from output
    let freed_bytes = parse_reclaimed_space(&stdout);

    if output.status.success() {
        Ok(DockerCleanResult {
            freed_bytes,
            containers_removed: 0, // Not easily parseable from output
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: true,
            message: format!("System prune completed. Reclaimed {} bytes", freed_bytes),
        })
    } else {
        Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: format!("System prune failed: {}", stderr.trim()),
        })
    }
}

/// Prune Docker builder cache
pub async fn docker_builder_prune() -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["builder", "prune", "-af"])
            .output()
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let freed_bytes = parse_reclaimed_space(&stdout);

    if output.status.success() {
        Ok(DockerCleanResult {
            freed_bytes,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: true,
            message: format!("Builder cache pruned. Reclaimed {} bytes", freed_bytes),
        })
    } else {
        Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: format!("Builder prune failed: {}", stderr.trim()),
        })
    }
}

/// Prune stopped containers
pub async fn prune_containers() -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["container", "prune", "-f"])
            .output()
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let freed_bytes = parse_reclaimed_space(&stdout);

    if output.status.success() {
        Ok(DockerCleanResult {
            freed_bytes,
            containers_removed: count_deleted_items(&stdout),
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: true,
            message: "Containers pruned successfully".to_string(),
        })
    } else {
        Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: format!("Container prune failed: {}", stderr.trim()),
        })
    }
}

/// Prune dangling images
pub async fn prune_images(all: bool) -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    let mut args = vec!["image", "prune", "-f"];
    if all {
        args.push("-a");
    }

    let output = tokio::task::spawn_blocking(move || {
        Command::new("docker").args(&args).output()
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let freed_bytes = parse_reclaimed_space(&stdout);

    if output.status.success() {
        Ok(DockerCleanResult {
            freed_bytes,
            containers_removed: 0,
            images_removed: count_deleted_items(&stdout),
            volumes_removed: 0,
            networks_removed: 0,
            success: true,
            message: "Images pruned successfully".to_string(),
        })
    } else {
        Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: format!("Image prune failed: {}", stderr.trim()),
        })
    }
}

/// Prune unused volumes
pub async fn prune_volumes() -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["volume", "prune", "-f"])
            .output()
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let freed_bytes = parse_reclaimed_space(&stdout);

    if output.status.success() {
        Ok(DockerCleanResult {
            freed_bytes,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: count_deleted_items(&stdout),
            networks_removed: 0,
            success: true,
            message: "Volumes pruned successfully".to_string(),
        })
    } else {
        Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: format!("Volume prune failed: {}", stderr.trim()),
        })
    }
}

/// Prune unused networks
pub async fn prune_networks() -> Result<DockerCleanResult> {
    if !is_docker_running().await {
        return Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: "Docker daemon is not running".to_string(),
        });
    }

    let output = tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .args(["network", "prune", "-f"])
            .output()
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: count_deleted_items(&stdout),
            success: true,
            message: "Networks pruned successfully".to_string(),
        })
    } else {
        Ok(DockerCleanResult {
            freed_bytes: 0,
            containers_removed: 0,
            images_removed: 0,
            volumes_removed: 0,
            networks_removed: 0,
            success: false,
            message: format!("Network prune failed: {}", stderr.trim()),
        })
    }
}

// ============================================================================
// Smart Suggestion Functions
// ============================================================================

/// Generate smart suggestions for Docker cleanup
pub async fn get_docker_suggestions() -> Result<Vec<DockerSuggestion>> {
    let scan_result = scan_docker_resources().await?;
    
    if !scan_result.daemon_running {
        return Ok(vec![]);
    }

    let mut suggestions = Vec::new();

    // Suggest stopped containers
    for container in &scan_result.containers {
        if container.state != ContainerState::Running {
            let (score, reasons, auto_select) = score_container(container);
            suggestions.push(DockerSuggestion {
                resource_type: DockerResourceType::Container,
                id: container.id.clone(),
                name: container.name.clone(),
                size: container.size,
                score,
                reasons,
                auto_select,
            });
        }
    }

    // Suggest dangling and unused images
    for image in &scan_result.images {
        if image.is_dangling || image.used_by_containers.is_empty() {
            let (score, reasons, auto_select) = score_image(image);
            let name = if image.repository == "<none>" {
                format!("{}...", &image.id[..12.min(image.id.len())])
            } else {
                format!("{}:{}", image.repository, image.tag)
            };
            
            suggestions.push(DockerSuggestion {
                resource_type: DockerResourceType::Image,
                id: image.id.clone(),
                name,
                size: image.size,
                score,
                reasons,
                auto_select,
            });
        }
    }

    // Suggest orphan volumes
    for volume in &scan_result.volumes {
        if volume.used_by_containers.is_empty() {
            let (score, reasons, auto_select) = score_volume(volume);
            suggestions.push(DockerSuggestion {
                resource_type: DockerResourceType::Volume,
                id: volume.name.clone(),
                name: volume.name.clone(),
                size: volume.size.unwrap_or(0),
                score,
                reasons,
                auto_select,
            });
        }
    }

    // Suggest unused networks (excluding defaults)
    for network in &scan_result.networks {
        if network.used_by_containers.is_empty() 
            && !DEFAULT_NETWORKS.contains(&network.name.as_str()) 
        {
            let (score, reasons, auto_select) = score_network(network);
            suggestions.push(DockerSuggestion {
                resource_type: DockerResourceType::Network,
                id: network.id.clone(),
                name: network.name.clone(),
                size: 0,
                score,
                reasons,
                auto_select,
            });
        }
    }

    // Add build cache suggestion if there's any
    if scan_result.build_cache_size > 0 {
        suggestions.push(DockerSuggestion {
            resource_type: DockerResourceType::BuildCache,
            id: "build_cache".to_string(),
            name: "Docker Build Cache".to_string(),
            size: scan_result.build_cache_size,
            score: 0.8,
            reasons: vec![
                format!("Build cache: {}", format_size(scan_result.build_cache_size)),
                "Can be safely removed".to_string(),
            ],
            auto_select: false,
        });
    }

    // Sort by score descending
    suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    Ok(suggestions)
}

/// Score a container for cleanup suggestion
fn score_container(container: &DockerContainer) -> (f64, Vec<String>, bool) {
    let mut reasons = Vec::new();
    let mut score: f64 = 0.0;
    
    // Size score
    let size_score = calculate_size_score(container.size);
    score += size_score * SIZE_WEIGHT;
    
    if container.size >= SIZE_LARGE {
        reasons.push(format!("Large size: {}", format_size(container.size)));
    } else if container.size >= SIZE_MEDIUM {
        reasons.push(format!("Size: {}", format_size(container.size)));
    }

    // State score - stopped containers get higher scores
    match container.state {
        ContainerState::Exited => {
            score += USAGE_WEIGHT; // Full usage weight since it's not running
            reasons.push("Container has exited".to_string());
        }
        ContainerState::Dead => {
            score += USAGE_WEIGHT;
            reasons.push("Container is dead".to_string());
        }
        ContainerState::Created => {
            score += USAGE_WEIGHT * 0.5;
            reasons.push("Container was created but never started".to_string());
        }
        _ => {}
    }

    // Age score based on status string (e.g., "Exited (0) 2 weeks ago")
    let age_score = estimate_age_score(&container.status);
    score += age_score * AGE_WEIGHT;
    
    if age_score >= 0.8 {
        reasons.push("Not used for a long time".to_string());
    }

    // Auto-select dead containers
    let auto_select = container.state == ContainerState::Dead;

    (score.min(1.0), reasons, auto_select)
}

/// Score an image for cleanup suggestion
fn score_image(image: &DockerImage) -> (f64, Vec<String>, bool) {
    let mut reasons = Vec::new();
    let mut score: f64 = 0.0;
    let mut auto_select = false;

    // Dangling images always get high score and auto-select
    if image.is_dangling {
        score = 1.0;
        reasons.push("Dangling image (untagged)".to_string());
        auto_select = true;
    } else {
        // Unused images
        if image.used_by_containers.is_empty() {
            score += USAGE_WEIGHT;
            reasons.push("Not used by any container".to_string());
        }

        // Size score
        let size_score = calculate_size_score(image.size);
        score += size_score * SIZE_WEIGHT;
    }

    if image.size >= SIZE_LARGE {
        reasons.push(format!("Large size: {}", format_size(image.size)));
    } else if image.size >= SIZE_MEDIUM {
        reasons.push(format!("Size: {}", format_size(image.size)));
    }

    (score.min(1.0), reasons, auto_select)
}

/// Score a volume for cleanup suggestion
fn score_volume(volume: &DockerVolume) -> (f64, Vec<String>, bool) {
    let mut reasons = Vec::new();
    let mut score: f64 = 0.0;

    // Orphan volumes (not used by any container) get high score
    if volume.used_by_containers.is_empty() {
        score = 1.0;
        reasons.push("Orphan volume (not used by any container)".to_string());
    }

    // Size score if available
    if let Some(size) = volume.size {
        if size >= SIZE_LARGE {
            reasons.push(format!("Large size: {}", format_size(size)));
        } else if size >= SIZE_MEDIUM {
            reasons.push(format!("Size: {}", format_size(size)));
        }
    }

    // Orphan volumes should be auto-selected
    let auto_select = volume.used_by_containers.is_empty();

    (score.min(1.0), reasons, auto_select)
}

/// Score a network for cleanup suggestion
fn score_network(network: &DockerNetwork) -> (f64, Vec<String>, bool) {
    let mut reasons = Vec::new();
    
    // Unused networks get high score
    let score = if network.used_by_containers.is_empty() {
        reasons.push("Not used by any container".to_string());
        0.9
    } else {
        0.3
    };

    // Don't auto-select networks (they don't take space and might be intentional)
    (score, reasons, false)
}

/// Calculate size score (0.0 - 1.0)
fn calculate_size_score(size: u64) -> f64 {
    if size >= SIZE_LARGE {
        1.0
    } else if size >= SIZE_MEDIUM {
        0.7
    } else if size >= SIZE_SMALL {
        0.4
    } else {
        (size as f64 / SIZE_SMALL as f64) * 0.4
    }
}

/// Estimate age score from Docker status string
fn estimate_age_score(status: &str) -> f64 {
    let lower = status.to_lowercase();
    
    // Parse common Docker time patterns
    if lower.contains("months") || lower.contains("year") {
        1.0
    } else if lower.contains("weeks") {
        let weeks = extract_number(&lower, "weeks").unwrap_or(1);
        if weeks >= 4 {
            0.9
        } else if weeks >= 2 {
            0.6
        } else {
            0.3
        }
    } else if lower.contains("days") {
        let days = extract_number(&lower, "days").unwrap_or(1);
        if days >= 30 {
            0.8
        } else if days >= 7 {
            0.5
        } else {
            0.2
        }
    } else if lower.contains("hours") {
        0.1
    } else {
        0.0
    }
}

/// Extract a number before a unit from a string (e.g., "2 weeks" -> 2)
fn extract_number(s: &str, unit: &str) -> Option<u64> {
    if let Some(pos) = s.find(unit) {
        let before = &s[..pos].trim();
        let parts: Vec<&str> = before.split_whitespace().collect();
        if let Some(num_str) = parts.last() {
            return num_str.parse().ok();
        }
    }
    None
}

/// Format size in human-readable form
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Clean Docker resources based on suggestions
pub async fn clean_docker_suggestions(suggestions: Vec<DockerSuggestion>) -> Result<DockerCleanResult> {
    let mut total_freed = 0u64;
    let mut containers_removed = 0;
    let mut images_removed = 0;
    let mut volumes_removed = 0;
    let mut networks_removed = 0;
    let mut errors = Vec::new();

    // Group suggestions by type
    let mut container_ids = Vec::new();
    let mut image_ids = Vec::new();
    let mut volume_names = Vec::new();
    let mut network_ids = Vec::new();
    let mut has_build_cache = false;

    for suggestion in &suggestions {
        match suggestion.resource_type {
            DockerResourceType::Container => container_ids.push(suggestion.id.clone()),
            DockerResourceType::Image => image_ids.push(suggestion.id.clone()),
            DockerResourceType::Volume => volume_names.push(suggestion.id.clone()),
            DockerResourceType::Network => network_ids.push(suggestion.id.clone()),
            DockerResourceType::BuildCache => has_build_cache = true,
        }
    }

    // Remove containers first (images might depend on them)
    if !container_ids.is_empty() {
        match remove_containers(container_ids, true).await {
            Ok(result) => {
                containers_removed = result.containers_removed;
                total_freed += result.freed_bytes;
                if !result.success {
                    errors.push(result.message);
                }
            }
            Err(e) => errors.push(format!("Container removal error: {}", e)),
        }
    }

    // Remove images
    if !image_ids.is_empty() {
        match remove_images(image_ids, true).await {
            Ok(result) => {
                images_removed = result.images_removed;
                total_freed += result.freed_bytes;
                if !result.success {
                    errors.push(result.message);
                }
            }
            Err(e) => errors.push(format!("Image removal error: {}", e)),
        }
    }

    // Remove volumes
    if !volume_names.is_empty() {
        match remove_volumes(volume_names).await {
            Ok(result) => {
                volumes_removed = result.volumes_removed;
                total_freed += result.freed_bytes;
                if !result.success {
                    errors.push(result.message);
                }
            }
            Err(e) => errors.push(format!("Volume removal error: {}", e)),
        }
    }

    // Remove networks
    if !network_ids.is_empty() {
        match remove_networks(network_ids).await {
            Ok(result) => {
                networks_removed = result.networks_removed;
                if !result.success {
                    errors.push(result.message);
                }
            }
            Err(e) => errors.push(format!("Network removal error: {}", e)),
        }
    }

    // Prune build cache
    if has_build_cache {
        match docker_builder_prune().await {
            Ok(result) => {
                total_freed += result.freed_bytes;
                if !result.success {
                    errors.push(result.message);
                }
            }
            Err(e) => errors.push(format!("Build cache prune error: {}", e)),
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!(
            "Cleanup complete: {} containers, {} images, {} volumes, {} networks removed. {} freed.",
            containers_removed, images_removed, volumes_removed, networks_removed,
            format_size(total_freed)
        )
    } else {
        format!(
            "Partial cleanup: {} containers, {} images, {} volumes, {} networks. Errors: {}",
            containers_removed, images_removed, volumes_removed, networks_removed,
            errors.join("; ")
        )
    };

    Ok(DockerCleanResult {
        freed_bytes: total_freed,
        containers_removed,
        images_removed,
        volumes_removed,
        networks_removed,
        success,
        message,
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse Docker size string (e.g., "1.5GB", "500MB", "100kB") to bytes
fn parse_docker_size(size_str: &str) -> u64 {
    let size_str = size_str.trim();
    
    // Handle "0B" or empty
    if size_str.is_empty() || size_str == "0B" || size_str == "0" {
        return 0;
    }

    // Extract number and unit
    let mut num_str = String::new();
    let mut unit_str = String::new();
    let mut in_unit = false;

    for c in size_str.chars() {
        if c.is_ascii_digit() || c == '.' {
            if !in_unit {
                num_str.push(c);
            }
        } else if c.is_alphabetic() {
            in_unit = true;
            unit_str.push(c);
        }
    }

    let num: f64 = num_str.parse().unwrap_or(0.0);
    let unit = unit_str.to_uppercase();

    let multiplier: u64 = match unit.as_str() {
        "B" => 1,
        "KB" | "K" => 1024,
        "MB" | "M" => 1024 * 1024,
        "GB" | "G" => 1024 * 1024 * 1024,
        "TB" | "T" => 1024 * 1024 * 1024 * 1024,
        _ => 1,
    };

    (num * multiplier as f64) as u64
}

/// Parse reclaimed space from Docker prune output
fn parse_reclaimed_space(output: &str) -> u64 {
    // Look for patterns like "Total reclaimed space: 1.5GB"
    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains("reclaimed") || lower.contains("freed") {
            // Find the size part
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                return parse_docker_size(parts[1].trim());
            }
            // Try to find size pattern in the line
            for word in line.split_whitespace() {
                if word.chars().any(|c| c.is_ascii_digit()) 
                    && word.chars().any(|c| c.is_alphabetic()) 
                {
                    let size = parse_docker_size(word);
                    if size > 0 {
                        return size;
                    }
                }
            }
        }
    }
    0
}

/// Count deleted items from prune output
fn count_deleted_items(output: &str) -> usize {
    let mut count = 0;
    for line in output.lines() {
        let lower = line.to_lowercase();
        // Lines starting with "Deleted" or containing IDs
        if lower.starts_with("deleted") || line.starts_with("sha256:") {
            count += 1;
        }
    }
    count
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_size() {
        assert_eq!(parse_docker_size("0B"), 0);
        assert_eq!(parse_docker_size("100B"), 100);
        assert_eq!(parse_docker_size("1KB"), 1024);
        assert_eq!(parse_docker_size("1.5KB"), 1536);
        assert_eq!(parse_docker_size("1MB"), 1024 * 1024);
        assert_eq!(parse_docker_size("1.5MB"), (1.5 * 1024.0 * 1024.0) as u64);
        assert_eq!(parse_docker_size("1GB"), 1024 * 1024 * 1024);
        assert_eq!(parse_docker_size("2.5GB"), (2.5 * 1024.0 * 1024.0 * 1024.0) as u64);
    }

    #[test]
    fn test_container_state_from_str() {
        assert_eq!(ContainerState::from("running"), ContainerState::Running);
        assert_eq!(ContainerState::from("exited"), ContainerState::Exited);
        assert_eq!(ContainerState::from("created"), ContainerState::Created);
        assert_eq!(ContainerState::from("paused"), ContainerState::Paused);
        assert_eq!(ContainerState::from("RUNNING"), ContainerState::Running);
        assert_eq!(ContainerState::from("unknown_state"), ContainerState::Unknown);
    }

    #[test]
    fn test_parse_reclaimed_space() {
        assert_eq!(
            parse_reclaimed_space("Total reclaimed space: 1.5GB"),
            (1.5 * 1024.0 * 1024.0 * 1024.0) as u64
        );
        assert_eq!(
            parse_reclaimed_space("Reclaimed space: 500MB"),
            500 * 1024 * 1024
        );
        assert_eq!(parse_reclaimed_space("No space reclaimed"), 0);
    }

    #[test]
    fn test_count_deleted_items() {
        let output = "Deleted: abc123\nDeleted: def456\nsha256:789xyz\nTotal: 3";
        assert_eq!(count_deleted_items(output), 3);
    }

    #[test]
    fn test_default_networks_protection() {
        assert!(DEFAULT_NETWORKS.contains(&"bridge"));
        assert!(DEFAULT_NETWORKS.contains(&"host"));
        assert!(DEFAULT_NETWORKS.contains(&"none"));
        assert!(!DEFAULT_NETWORKS.contains(&"custom_network"));
    }

    #[test]
    fn test_docker_container_serialization() {
        let container = DockerContainer {
            id: "abc123".to_string(),
            name: "test-container".to_string(),
            image: "nginx:latest".to_string(),
            status: "Exited (0) 2 days ago".to_string(),
            state: ContainerState::Exited,
            size: 1024 * 1024,
            created: "2024-01-01".to_string(),
            ports: "80/tcp".to_string(),
        };

        let json = serde_json::to_string(&container).unwrap();
        assert!(json.contains("abc123"));
        assert!(json.contains("test-container"));
        
        let deserialized: DockerContainer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, container.id);
        assert_eq!(deserialized.state, ContainerState::Exited);
    }

    #[test]
    fn test_docker_image_serialization() {
        let image = DockerImage {
            id: "sha256:abc123".to_string(),
            repository: "nginx".to_string(),
            tag: "latest".to_string(),
            size: 100 * 1024 * 1024,
            created: "2024-01-01".to_string(),
            is_dangling: false,
            used_by_containers: vec!["container1".to_string()],
        };

        let json = serde_json::to_string(&image).unwrap();
        let deserialized: DockerImage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.repository, "nginx");
        assert!(!deserialized.is_dangling);
    }

    #[test]
    fn test_docker_scan_result_serialization() {
        let result = DockerScanResult {
            daemon_running: true,
            containers: vec![],
            images: vec![],
            volumes: vec![],
            networks: vec![],
            build_cache_size: 1024,
            total_reclaimable: 2048,
            stopped_containers_count: 5,
            dangling_images_count: 3,
            unused_images_count: 10,
            orphan_volumes_count: 2,
            unused_networks_count: 1,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("daemon_running"));
        assert!(json.contains("total_reclaimable"));
    }

    #[test]
    fn test_docker_clean_result_serialization() {
        let result = DockerCleanResult {
            freed_bytes: 1024 * 1024 * 100,
            containers_removed: 5,
            images_removed: 3,
            volumes_removed: 2,
            networks_removed: 1,
            success: true,
            message: "Cleanup successful".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: DockerCleanResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.containers_removed, 5);
        assert!(deserialized.success);
    }

    #[test]
    fn test_docker_suggestion_serialization() {
        let suggestion = DockerSuggestion {
            resource_type: DockerResourceType::Image,
            id: "sha256:abc".to_string(),
            name: "dangling-image".to_string(),
            size: 500 * 1024 * 1024,
            score: 0.95,
            reasons: vec!["Dangling image".to_string(), "Large size".to_string()],
            auto_select: true,
        };

        let json = serde_json::to_string(&suggestion).unwrap();
        let deserialized: DockerSuggestion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.resource_type, DockerResourceType::Image);
        assert!(deserialized.auto_select);
    }

    #[test]
    fn test_calculate_size_score() {
        assert_eq!(calculate_size_score(SIZE_LARGE), 1.0);
        assert_eq!(calculate_size_score(SIZE_LARGE + 1000), 1.0);
        assert_eq!(calculate_size_score(SIZE_MEDIUM), 0.7);
        assert_eq!(calculate_size_score(SIZE_SMALL), 0.4);
        assert!(calculate_size_score(SIZE_SMALL / 2) < 0.4);
        assert!(calculate_size_score(0) < 0.1);
    }

    #[test]
    fn test_estimate_age_score() {
        assert_eq!(estimate_age_score("Exited (0) 2 months ago"), 1.0);
        assert_eq!(estimate_age_score("Exited (0) 1 year ago"), 1.0);
        assert!(estimate_age_score("Exited (0) 4 weeks ago") >= 0.8);
        assert!(estimate_age_score("Exited (0) 2 weeks ago") >= 0.5);
        assert!(estimate_age_score("Exited (0) 30 days ago") >= 0.7);
        assert!(estimate_age_score("Exited (0) 7 days ago") >= 0.4);
        assert!(estimate_age_score("Exited (0) 2 hours ago") < 0.2);
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("2 weeks ago", "weeks"), Some(2));
        assert_eq!(extract_number("30 days ago", "days"), Some(30));
        assert_eq!(extract_number("3 months ago", "months"), Some(3));
        assert_eq!(extract_number("no number here", "weeks"), None);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 1024), "1.0 TB");
        assert_eq!(format_size((1.5 * 1024.0 * 1024.0 * 1024.0) as u64), "1.5 GB");
    }

    #[test]
    fn test_score_container_exited() {
        let container = DockerContainer {
            id: "abc123".to_string(),
            name: "test".to_string(),
            image: "nginx".to_string(),
            status: "Exited (0) 2 weeks ago".to_string(),
            state: ContainerState::Exited,
            size: SIZE_LARGE,
            created: "".to_string(),
            ports: "".to_string(),
        };

        let (score, reasons, auto_select) = score_container(&container);
        assert!(score >= 0.5);
        assert!(!reasons.is_empty());
        assert!(!auto_select); // Exited containers shouldn't auto-select
    }

    #[test]
    fn test_score_container_dead() {
        let container = DockerContainer {
            id: "abc123".to_string(),
            name: "test".to_string(),
            image: "nginx".to_string(),
            status: "Dead".to_string(),
            state: ContainerState::Dead,
            size: SIZE_SMALL,
            created: "".to_string(),
            ports: "".to_string(),
        };

        let (_, _, auto_select) = score_container(&container);
        assert!(auto_select); // Dead containers should auto-select
    }

    #[test]
    fn test_score_image_dangling() {
        let image = DockerImage {
            id: "sha256:abc".to_string(),
            repository: "<none>".to_string(),
            tag: "<none>".to_string(),
            size: SIZE_MEDIUM,
            created: "".to_string(),
            is_dangling: true,
            used_by_containers: vec![],
        };

        let (score, reasons, auto_select) = score_image(&image);
        assert_eq!(score, 1.0);
        assert!(auto_select);
        assert!(reasons.iter().any(|r| r.contains("Dangling")));
    }

    #[test]
    fn test_score_image_unused() {
        let image = DockerImage {
            id: "sha256:abc".to_string(),
            repository: "nginx".to_string(),
            tag: "latest".to_string(),
            size: SIZE_LARGE,
            created: "".to_string(),
            is_dangling: false,
            used_by_containers: vec![],
        };

        let (score, reasons, auto_select) = score_image(&image);
        assert!(score >= 0.5);
        assert!(!auto_select); // Non-dangling unused images shouldn't auto-select
        assert!(reasons.iter().any(|r| r.contains("Not used")));
    }

    #[test]
    fn test_score_volume_orphan() {
        let volume = DockerVolume {
            name: "test-volume".to_string(),
            driver: "local".to_string(),
            mountpoint: "/var/lib/docker/volumes/test".to_string(),
            size: Some(SIZE_LARGE),
            used_by_containers: vec![],
        };

        let (score, reasons, auto_select) = score_volume(&volume);
        assert_eq!(score, 1.0);
        assert!(auto_select);
        assert!(reasons.iter().any(|r| r.contains("Orphan")));
    }

    #[test]
    fn test_score_network_unused() {
        let network = DockerNetwork {
            id: "abc123".to_string(),
            name: "custom-network".to_string(),
            driver: "bridge".to_string(),
            scope: "local".to_string(),
            used_by_containers: vec![],
        };

        let (score, reasons, auto_select) = score_network(&network);
        assert!(score >= 0.8);
        assert!(!auto_select); // Networks shouldn't auto-select
        assert!(reasons.iter().any(|r| r.contains("Not used")));
    }
}

