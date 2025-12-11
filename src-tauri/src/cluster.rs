/// Unified Cluster Configuration Module
///
/// Single source of truth for all cluster-specific configuration including:
/// - Cluster capabilities (partitions, QoS, presets, billing)
/// - Resource limits and validation rules
/// - Cost calculations and queue time estimates
///
use serde::{Deserialize, Serialize};

// ============================================================================
// Core Cluster Profile Types
// ============================================================================

/// Cluster capabilities exposed to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterCapabilities {
    pub partitions: Vec<PartitionSpec>,
    pub qos_options: Vec<QosSpec>,
    pub job_presets: Vec<JobPreset>,
    pub billing_rates: BillingRates,
    pub default_host: String,
}

// ============================================================================
// Partition Configuration
// ============================================================================

/// Partition specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionSpec {
    pub name: String,
    pub title: String,
    pub description: String,
    pub max_cores: u32,
    pub max_memory_per_core_gb: f64,
    pub gpu_type: Option<String>,
    pub gpu_count: Option<u32>,
    /// Indicates if this is the default/recommended partition for new users
    pub is_default: bool,
}

// ============================================================================
// QoS Configuration
// ============================================================================

/// Quality of Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosSpec {
    pub name: String,
    pub title: String,
    pub description: String,
    pub max_walltime_hours: u32,
    pub valid_partitions: Vec<String>,
    pub min_memory_gb: Option<u32>,
    /// Indicates if this is the default QoS option
    pub is_default: bool,
}

// ============================================================================
// Job Presets
// ============================================================================

/// Job preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPreset {
    pub name: String,
    pub description: String,
    pub cores: u32,
    pub memory: String,
    pub walltime: String,
    pub partition: String,
    pub qos: String,
}

// ============================================================================
// Billing
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingRates {
    pub cpu_cost_per_core_hour: f64,
    pub gpu_cost_per_gpu_hour: f64,
}

// ============================================================================
// Server Interaction Timeouts
// ============================================================================

/// Server interaction timeout constants (in seconds)
/// These values are optimized for cluster etiquette and reliability
pub mod timeouts {
    /// Default SSH command timeout (2 minutes)
    pub const DEFAULT_COMMAND: u64 = 120;

    /// SLURM operations (1 minute) - cluster commands that query scheduler
    pub const SLURM_OPERATION: u64 = 60;

    /// Quick file operations (30 seconds) - file copy, directory creation
    pub const QUICK_OPERATION: u64 = 30;

    /// File copy operations (1 minute) - larger file transfers
    pub const FILE_COPY: u64 = 60;

    /// Quick status checks (10 seconds) - simple existence tests
    pub const STATUS_CHECK: u64 = 10;

    /// Job submission (30 seconds) - SLURM sbatch command
    pub const JOB_SUBMIT: u64 = 30;
}

// ============================================================================
// Embedded Configuration - Parse alpine.json
// ============================================================================

#[cfg(test)]
fn load_default_config_for_tests() -> ClusterCapabilities {
    const ALPINE_JSON: &str = include_str!("../cluster/alpine.json");
    serde_json::from_str(ALPINE_JSON).expect("Failed to parse alpine.json")
}

// ============================================================================
// Active Cluster Management
// ============================================================================
// All cluster data loaded from database (seeded from alpine.json)
// No hardcoded runtime data - fail fast if cache not populated

use std::sync::RwLock;
use lazy_static::lazy_static;

lazy_static! {
    /// Cached cluster capabilities loaded from database
    /// Initialized on app startup, updated when config changes
    static ref CLUSTER_CONFIG_CACHE: RwLock<Option<ClusterCapabilities>> = RwLock::new(None);
}

/// Set cluster config in cache (called after loading from DB or saving changes)
pub fn set_cluster_config_cache(config: ClusterCapabilities) {
    let mut cache = CLUSTER_CONFIG_CACHE.write().unwrap();
    *cache = Some(config);
}

/// Get partition by name from cached config
pub fn get_partition_by_name(partition_name: &str) -> Option<PartitionSpec> {
    let cache = CLUSTER_CONFIG_CACHE.read().unwrap();
    cache.as_ref()
        .and_then(|config| config.partitions.iter().find(|p| p.name == partition_name).cloned())
}

/// Get QoS options valid for a specific partition
pub fn get_qos_for_partition(partition_name: &str) -> Vec<QosSpec> {
    let cache = CLUSTER_CONFIG_CACHE.read().unwrap();
    match cache.as_ref() {
        Some(config) => config.qos_options.iter()
            .filter(|qos| qos.valid_partitions.contains(&partition_name.to_string()))
            .cloned()
            .collect(),
        None => vec![],
    }
}

/// Get QoS by name from cached config
pub fn get_qos_by_name(qos_name: &str) -> Option<QosSpec> {
    let cache = CLUSTER_CONFIG_CACHE.read().unwrap();
    cache.as_ref()
        .and_then(|config| config.qos_options.iter().find(|q| q.name == qos_name).cloned())
}

// ============================================================================
// Public Business Logic (called by commands/cluster.rs)
// ============================================================================

/// Get cluster capabilities for frontend
/// Panics if cache not initialized (call initialize_app first)
pub fn get_cluster_capabilities() -> ClusterCapabilities {
    CLUSTER_CONFIG_CACHE
        .read()
        .unwrap()
        .as_ref()
        .expect("Cluster config not initialized - initialize_app must be called first")
        .clone()
}

/// Save cluster config to database and update cache
pub fn save_cluster_config(config: ClusterCapabilities) -> crate::types::ApiResult<()> {
    use crate::{log_info, log_error};
    use crate::database::with_database;

    // Save to database
    match with_database(|db| db.save_cluster_config(&config)) {
        Ok(_) => {
            // Update cache
            set_cluster_config_cache(config);

            log_info!(
                category: "ClusterConfig",
                message: "Cluster config saved successfully",
                show_toast: true
            );

            crate::types::ApiResult::success(())
        }
        Err(e) => {
            log_error!(
                category: "ClusterConfig",
                message: "Failed to save cluster config",
                details: "{}",e
            );
            crate::types::ApiResult::error(format!("Failed to save cluster config: {}", e))
        }
    }
}

/// Reset cluster config to defaults (clears DB, re-seeds from embedded JSON)
pub fn reset_cluster_config() -> crate::types::ApiResult<ClusterCapabilities> {
    use crate::{log_info, log_error};
    use crate::database::with_database;

    // Delete current config from database
    match with_database(|db| db.delete_cluster_config()) {
        Ok(_) => {},
        Err(e) => {
            log_error!(
                category: "ClusterConfig",
                message: "Failed to clear cluster config",
                details: "{}", e
            );
            return crate::types::ApiResult::error(format!("Failed to clear cluster config: {}", e));
        }
    }

    // Re-seed from embedded JSON
    match crate::database::with_database(crate::database::load_default_cluster_config) {
        Ok(_) => {},
        Err(e) => {
            log_error!(
                category: "ClusterConfig",
                message: "Failed to re-seed cluster config",
                details: "{}", e
            );
            return crate::types::ApiResult::error(format!("Failed to re-seed cluster config: {}", e));
        }
    }

    // Load fresh config and update cache
    match with_database(|db| db.load_cluster_config()) {
        Ok(Some(config)) => {
            set_cluster_config_cache(config.clone());

            log_info!(
                category: "ClusterConfig",
                message: "Cluster config reset to defaults",
                show_toast: true
            );

            crate::types::ApiResult::success(config)
        }
        Ok(None) => {
            // Shouldn't happen - we just seeded
            log_error!(category: "ClusterConfig", message: "Config missing after reset");
            crate::types::ApiResult::error("Config missing after reset".to_string())
        }
        Err(e) => {
            log_error!(category: "ClusterConfig", message: "Failed to load after reset", details: "{}", e);
            crate::types::ApiResult::error(format!("Failed to load after reset: {}", e))
        }
    }
}

/// Calculate estimated job cost
pub fn calculate_job_cost(cores: u32, walltime: String, has_gpu: bool, gpu_count: u32) -> u32 {
    // Parse walltime string to hours
    let walltime_hours = match parse_walltime_to_hours(&walltime) {
        Ok(hours) => hours,
        Err(_) => return 0, // Return 0 cost if walltime is invalid
    };

    // Get billing rates from cached config
    let billing = {
        let cache = CLUSTER_CONFIG_CACHE.read().unwrap();
        cache.as_ref()
            .map(|config| config.billing_rates.clone())
            .expect("Cluster config not initialized")
    };

    let core_cost = cores as f64 * walltime_hours * billing.cpu_cost_per_core_hour;
    let gpu_cost = if has_gpu {
        gpu_count as f64 * walltime_hours * billing.gpu_cost_per_gpu_hour
    } else {
        0.0
    };
    (core_cost + gpu_cost).round() as u32
}

/// Helper function to parse walltime string (HH:MM:SS) to hours
fn parse_walltime_to_hours(walltime: &str) -> anyhow::Result<f64> {
    if walltime.trim().is_empty() {
        return Err(anyhow::anyhow!("Walltime is required"));
    }

    let parts: Vec<&str> = walltime.split(':').collect();

    if parts.len() != 3 {
        return Err(anyhow::anyhow!("Walltime must be in HH:MM:SS format"));
    }

    let hours: u32 = parts[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid hours in walltime"))?;
    let minutes: u32 = parts[1].parse()
        .map_err(|_| anyhow::anyhow!("Invalid minutes in walltime"))?;
    let seconds: u32 = parts[2].parse()
        .map_err(|_| anyhow::anyhow!("Invalid seconds in walltime"))?;

    if minutes >= 60 {
        return Err(anyhow::anyhow!("Minutes must be less than 60"));
    }
    if seconds >= 60 {
        return Err(anyhow::anyhow!("Seconds must be less than 60"));
    }

    Ok(hours as f64 + (minutes as f64 / 60.0) + (seconds as f64 / 3600.0))
}

/// Estimate queue time based on resources and partition
pub fn estimate_queue_time(cores: u32, partition_id: String) -> String {
    let partition_id = partition_id.as_str();
    // GPU partitions generally have longer queues
    if partition_id.contains("a100") || partition_id.contains("mi100") || partition_id.contains("l40") {
        return match cores {
            0..=32 => "1-4 hours",
            33..=64 => "4-8 hours",
            _ => "> 8 hours",
        }.to_string();
    }

    // Testing partitions are fast
    if partition_id.starts_with("atesting") || partition_id == "acompile" {
        return "< 15 minutes".to_string();
    }

    // Standard CPU partitions
    match cores {
        0..=24 => "< 30 minutes",
        25..=48 => "< 2 hours",
        49..=128 => "2-6 hours",
        _ => "> 6 hours",
    }.to_string()
}

/// Suggest optimal QOS based on walltime and partition
pub fn suggest_qos(walltime_hours: f64, partition_id: String) -> String {
    let partition_id = partition_id.as_str();
    if partition_id == "amem" {
        "mem".to_string()
    } else if partition_id.starts_with("atesting") {
        "testing".to_string()
    } else if partition_id == "acompile" {
        "compile".to_string()
    } else if walltime_hours > 24.0 {
        let available_qos = get_qos_for_partition(partition_id);
        if available_qos.iter().any(|q| q.name == "long") {
            "long".to_string()
        } else {
            "normal".to_string()
        }
    } else {
        "normal".to_string()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpine_has_all_partitions() {
        let capabilities = load_default_config_for_tests();
        let partitions = &capabilities.partitions;

        assert!(partitions.iter().any(|p| p.name == "amilan"));
        assert!(partitions.iter().any(|p| p.name == "amilan128c"));
        assert!(partitions.iter().any(|p| p.name == "amem"));
        assert!(partitions.iter().any(|p| p.name == "aa100"));
        assert!(partitions.iter().any(|p| p.name == "ami100"));
        assert!(partitions.iter().any(|p| p.name == "al40"));
        assert!(partitions.iter().any(|p| p.name == "atesting"));
        assert!(partitions.iter().any(|p| p.name == "atesting_a100"));
        assert!(partitions.iter().any(|p| p.name == "atesting_mi100"));
        assert!(partitions.iter().any(|p| p.name == "acompile"));
    }

    #[test]
    fn test_alpine_has_default_partition() {
        let capabilities = load_default_config_for_tests();
        let default_partition = capabilities.partitions.iter()
            .find(|p| p.is_default);

        assert!(default_partition.is_some());
        assert_eq!(default_partition.unwrap().name, "amilan");
    }

    #[test]
    fn test_alpine_has_all_qos() {
        let capabilities = load_default_config_for_tests();
        let qos = &capabilities.qos_options;

        assert!(qos.iter().any(|q| q.name == "normal"));
        assert!(qos.iter().any(|q| q.name == "long"));
        assert!(qos.iter().any(|q| q.name == "mem"));
        assert!(qos.iter().any(|q| q.name == "testing"));
        assert!(qos.iter().any(|q| q.name == "compile"));
    }

    #[test]
    fn test_alpine_has_default_qos() {
        let capabilities = load_default_config_for_tests();
        let default_qos = capabilities.qos_options.iter()
            .find(|q| q.is_default);

        assert!(default_qos.is_some());
        assert_eq!(default_qos.unwrap().name, "normal");
    }

    #[test]
    fn test_get_qos_for_partition() {
        // Setup: populate cache with test data
        set_cluster_config_cache(load_default_config_for_tests());

        let amilan_qos = get_qos_for_partition("amilan");
        assert!(amilan_qos.iter().any(|q| q.name == "normal"));
        assert!(amilan_qos.iter().any(|q| q.name == "long"));
        assert!(!amilan_qos.iter().any(|q| q.name == "mem"));

        let amem_qos = get_qos_for_partition("amem");
        assert!(amem_qos.iter().any(|q| q.name == "mem"));
        assert!(!amem_qos.iter().any(|q| q.name == "normal"));
    }

    #[test]
    fn test_calculate_job_cost() {
        // Setup: populate cache with test data
        set_cluster_config_cache(load_default_config_for_tests());

        // CPU only: 24 cores * 4 hours = 96 SU
        assert_eq!(calculate_job_cost(24, "04:00:00".to_string(), false, 0), 96);

        // With GPU: (64 cores * 24 hours) + (1 GPU * 24 hours * 108.2) = 1536 + 2596.8 = 4133
        let cost = calculate_job_cost(64, "24:00:00".to_string(), true, 1);
        assert!((4130..=4140).contains(&cost));
    }

    #[test]
    fn test_estimate_queue_time() {
        assert_eq!(estimate_queue_time(24, "amilan".to_string()), "< 30 minutes");
        assert_eq!(estimate_queue_time(48, "amilan".to_string()), "< 2 hours");
        assert_eq!(estimate_queue_time(128, "amilan128c".to_string()), "2-6 hours");
        assert_eq!(estimate_queue_time(32, "aa100".to_string()), "1-4 hours");
        assert_eq!(estimate_queue_time(8, "atesting".to_string()), "< 15 minutes");
    }

    #[test]
    fn test_suggest_qos() {
        // Setup: populate cache with test data
        set_cluster_config_cache(load_default_config_for_tests());

        assert_eq!(suggest_qos(12.0, "amilan".to_string()), "normal");
        assert_eq!(suggest_qos(48.0, "amilan".to_string()), "long");
        assert_eq!(suggest_qos(24.0, "amem".to_string()), "mem");
        assert_eq!(suggest_qos(0.5, "atesting".to_string()), "testing");
        assert_eq!(suggest_qos(4.0, "acompile".to_string()), "compile");
    }

}
