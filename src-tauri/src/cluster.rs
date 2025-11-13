/// Unified Cluster Configuration Module
///
/// Single source of truth for all cluster-specific configuration including:
/// - Connection settings (hostname, port, module setup)
/// - Cluster capabilities (partitions, QoS, presets, billing)
/// - Resource limits and validation rules
///
/// Provides a unified ClusterProfile concept, making the system ready for future 
/// multi-cluster support while keeping current implementation simple.
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

// ============================================================================
// Core Cluster Profile Types
// ============================================================================

/// Complete cluster profile including connection and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterProfile {
    pub id: String,
    pub connection: ConnectionConfig,
    pub capabilities: ClusterCapabilities,
}

/// SSH connection configuration for a cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Cluster name for identification
    pub name: String,
    /// Login server hostname
    pub login_server: String,
    /// SSH port (typically 22)
    pub port: u16,
    /// Module setup command (e.g., "source /etc/profile && module load slurm/alpine")
    pub module_setup: String,
}

/// Cluster capabilities exposed to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterCapabilities {
    pub partitions: Vec<PartitionSpec>,
    pub qos_options: Vec<QosSpec>,
    pub job_presets: Vec<JobPreset>,
    pub billing_rates: BillingRates,
}

// ============================================================================
// Partition Configuration
// ============================================================================

/// Partition category classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PartitionCategory {
    Compute,
    #[serde(rename = "GPU")]
    Gpu,
    HighMemory,
    Development,
    Compile,
}

/// Partition specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionSpec {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub nodes: String,
    pub cores_per_node: String,
    pub ram_per_core: String,
    pub max_walltime: String,
    pub gpu_type: Option<String>,
    pub gpu_count: Option<u32>,
    pub category: PartitionCategory,
    pub use_cases: Vec<String>,
    pub is_standard: bool,
    /// Indicates if this is the default/recommended partition for new users
    pub is_default: bool,
}

/// Resource limits for a partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionLimits {
    pub max_cores: u32,
    pub max_memory_per_core: f64,
    pub min_memory_for_qos: Option<u32>,
}

// ============================================================================
// QoS Configuration
// ============================================================================

/// Quality of Service priority level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QosPriority {
    Normal,
    High,
    Low,
}

/// Quality of Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosSpec {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub max_walltime_hours: u32,
    pub max_jobs: u32,
    pub node_limit: u32,
    pub valid_partitions: Vec<String>,
    pub requirements: Vec<String>,
    pub priority: QosPriority,
    /// Indicates if this is the default QoS option
    pub is_default: bool,
}

// ============================================================================
// Job Presets
// ============================================================================

/// Job preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub config: JobPresetConfig,
    pub estimated_cost: String,
    pub estimated_queue: String,
    pub use_cases: Vec<String>,
    pub requires_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPresetConfig {
    pub cores: u32,
    pub memory: String,
    pub wall_time: String,
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
// Default Alpine Cluster Profile
// ============================================================================

impl Default for ClusterProfile {
    fn default() -> Self {
        alpine_profile()
    }
}

/// Get the Alpine cluster profile (CU Research Computing)
pub fn alpine_profile() -> ClusterProfile {
    ClusterProfile {
        id: "alpine".to_string(),
        connection: ConnectionConfig {
            name: "Alpine".to_string(),
            login_server: "login.rc.colorado.edu".to_string(),
            port: 22,
            module_setup: "source /etc/profile && module load slurm/alpine".to_string(),
        },
        capabilities: ClusterCapabilities {
            partitions: get_alpine_partitions(),
            qos_options: get_alpine_qos(),
            job_presets: get_alpine_presets(),
            billing_rates: BillingRates {
                cpu_cost_per_core_hour: 1.0,
                gpu_cost_per_gpu_hour: 108.2,
            },
        },
    }
}

/// Get all Alpine partitions (production + testing)
fn get_alpine_partitions() -> Vec<PartitionSpec> {
    vec![
        // Production partitions
        PartitionSpec {
            id: "amilan".to_string(),
            name: "amilan".to_string(),
            title: "General Compute (Default)".to_string(),
            description: "Standard CPU nodes for most NAMD simulations".to_string(),
            nodes: "374+".to_string(),
            cores_per_node: "32/48/64".to_string(),
            ram_per_core: "3.75 GB".to_string(),
            max_walltime: "24H (7D with long QoS)".to_string(),
            gpu_type: None,
            gpu_count: None,
            category: PartitionCategory::Compute,
            use_cases: vec![
                "Production runs".to_string(),
                "Standard simulations".to_string(),
                "Most NAMD jobs".to_string(),
            ],
            is_standard: true,
            is_default: true,
        },
        PartitionSpec {
            id: "amilan128c".to_string(),
            name: "amilan128c".to_string(),
            title: "High-Core Compute".to_string(),
            description: "High core count nodes for large parallel jobs".to_string(),
            nodes: "16+".to_string(),
            cores_per_node: "128".to_string(),
            ram_per_core: "2.01 GB".to_string(),
            max_walltime: "24H (7D with long QoS)".to_string(),
            gpu_type: None,
            gpu_count: None,
            category: PartitionCategory::Compute,
            use_cases: vec![
                "Large simulations".to_string(),
                "Highly parallel jobs".to_string(),
                "100+ core jobs".to_string(),
            ],
            is_standard: false,
            is_default: false,
        },
        PartitionSpec {
            id: "amem".to_string(),
            name: "amem".to_string(),
            title: "High-Memory".to_string(),
            description: "High memory nodes for memory-intensive simulations".to_string(),
            nodes: "22+".to_string(),
            cores_per_node: "48/64/128".to_string(),
            ram_per_core: "16-21.5 GB".to_string(),
            max_walltime: "4H (7D with mem QoS)".to_string(),
            gpu_type: None,
            gpu_count: None,
            category: PartitionCategory::HighMemory,
            use_cases: vec![
                "Large systems".to_string(),
                "Memory-intensive jobs".to_string(),
                "Complex simulations".to_string(),
            ],
            is_standard: false,
            is_default: false,
        },
        PartitionSpec {
            id: "aa100".to_string(),
            name: "aa100".to_string(),
            title: "NVIDIA A100 GPU".to_string(),
            description: "GPU-accelerated nodes with NVIDIA A100 for fast NAMD".to_string(),
            nodes: "10+".to_string(),
            cores_per_node: "64".to_string(),
            ram_per_core: "3.75 GB".to_string(),
            max_walltime: "24H (7D with long QoS)".to_string(),
            gpu_type: Some("NVIDIA A100".to_string()),
            gpu_count: Some(3),
            category: PartitionCategory::Gpu,
            use_cases: vec![
                "GPU-accelerated NAMD".to_string(),
                "Fast simulations".to_string(),
                "CUDA workloads".to_string(),
            ],
            is_standard: false,
            is_default: false,
        },
        PartitionSpec {
            id: "ami100".to_string(),
            name: "ami100".to_string(),
            title: "AMD MI100 GPU".to_string(),
            description: "GPU-accelerated nodes with AMD MI100 for HIP workloads".to_string(),
            nodes: "8+".to_string(),
            cores_per_node: "64".to_string(),
            ram_per_core: "3.75 GB".to_string(),
            max_walltime: "24H (7D with long QoS)".to_string(),
            gpu_type: Some("AMD MI100".to_string()),
            gpu_count: Some(3),
            category: PartitionCategory::Gpu,
            use_cases: vec![
                "HIP-accelerated workloads".to_string(),
                "AMD GPU computing".to_string(),
                "Alternative GPU option".to_string(),
            ],
            is_standard: false,
            is_default: false,
        },
        PartitionSpec {
            id: "al40".to_string(),
            name: "al40".to_string(),
            title: "NVIDIA L40 GPU".to_string(),
            description: "GPU-accelerated nodes with NVIDIA L40".to_string(),
            nodes: "2+".to_string(),
            cores_per_node: "64".to_string(),
            ram_per_core: "3.75 GB".to_string(),
            max_walltime: "24H (7D with long QoS)".to_string(),
            gpu_type: Some("NVIDIA L40".to_string()),
            gpu_count: Some(3),
            category: PartitionCategory::Gpu,
            use_cases: vec![
                "GPU computing".to_string(),
                "Graphics workloads".to_string(),
                "AI/ML tasks".to_string(),
            ],
            is_standard: false,
            is_default: false,
        },
        // Testing partitions
        PartitionSpec {
            id: "atesting".to_string(),
            name: "atesting".to_string(),
            title: "CPU Testing".to_string(),
            description: "Quick CPU testing with limited resources".to_string(),
            nodes: "2".to_string(),
            cores_per_node: "16 total".to_string(),
            ram_per_core: "Variable".to_string(),
            max_walltime: "1 hour".to_string(),
            gpu_type: None,
            gpu_count: None,
            category: PartitionCategory::Development,
            use_cases: vec![
                "Testing configurations".to_string(),
                "Quick validation".to_string(),
                "Development".to_string(),
            ],
            is_standard: false,
            is_default: false,
        },
        PartitionSpec {
            id: "atesting_a100".to_string(),
            name: "atesting_a100".to_string(),
            title: "GPU Testing (A100 MIG)".to_string(),
            description: "GPU testing with A100 MIG instances".to_string(),
            nodes: "1".to_string(),
            cores_per_node: "10".to_string(),
            ram_per_core: "Variable".to_string(),
            max_walltime: "1 hour".to_string(),
            gpu_type: Some("NVIDIA A100 MIG".to_string()),
            gpu_count: Some(1),
            category: PartitionCategory::Development,
            use_cases: vec![
                "GPU testing".to_string(),
                "CUDA development".to_string(),
                "Quick GPU validation".to_string(),
            ],
            is_standard: false,
            is_default: false,
        },
        PartitionSpec {
            id: "atesting_mi100".to_string(),
            name: "atesting_mi100".to_string(),
            title: "GPU Testing (MI100)".to_string(),
            description: "GPU testing with AMD MI100".to_string(),
            nodes: "1".to_string(),
            cores_per_node: "64".to_string(),
            ram_per_core: "3.75 GB".to_string(),
            max_walltime: "1 hour".to_string(),
            gpu_type: Some("AMD MI100".to_string()),
            gpu_count: Some(3),
            category: PartitionCategory::Development,
            use_cases: vec![
                "HIP testing".to_string(),
                "AMD GPU development".to_string(),
                "GPU validation".to_string(),
            ],
            is_standard: false,
            is_default: false,
        },
        PartitionSpec {
            id: "acompile".to_string(),
            name: "acompile".to_string(),
            title: "Code Compilation".to_string(),
            description: "Dedicated nodes for compiling code".to_string(),
            nodes: "1".to_string(),
            cores_per_node: "4".to_string(),
            ram_per_core: "Variable".to_string(),
            max_walltime: "12 hours".to_string(),
            gpu_type: None,
            gpu_count: None,
            category: PartitionCategory::Compile,
            use_cases: vec![
                "Code compilation".to_string(),
                "Build processes".to_string(),
                "Software development".to_string(),
            ],
            is_standard: false,
            is_default: false,
        },
    ]
}

/// Get Alpine QoS options
fn get_alpine_qos() -> Vec<QosSpec> {
    vec![
        QosSpec {
            id: "normal".to_string(),
            name: "normal".to_string(),
            title: "Normal Priority (Default)".to_string(),
            description: "Standard priority with good resource limits".to_string(),
            max_walltime_hours: 24,
            max_jobs: 1000,
            node_limit: 128,
            valid_partitions: vec![
                "amilan".to_string(),
                "amilan128c".to_string(),
                "aa100".to_string(),
                "ami100".to_string(),
                "al40".to_string(),
            ],
            requirements: vec![],
            priority: QosPriority::Normal,
            is_default: true,
        },
        QosSpec {
            id: "long".to_string(),
            name: "long".to_string(),
            title: "Extended Runtime".to_string(),
            description: "Longer walltime for extended simulations".to_string(),
            max_walltime_hours: 168, // 7 days
            max_jobs: 200,
            node_limit: 20,
            valid_partitions: vec![
                "amilan".to_string(),
                "amilan128c".to_string(),
                "aa100".to_string(),
                "ami100".to_string(),
                "al40".to_string(),
            ],
            requirements: vec![],
            priority: QosPriority::Normal,
            is_default: false,
        },
        QosSpec {
            id: "mem".to_string(),
            name: "mem".to_string(),
            title: "High-Memory".to_string(),
            description: "For high-memory jobs on amem partition".to_string(),
            max_walltime_hours: 168, // 7 days
            max_jobs: 1000,
            node_limit: 12,
            valid_partitions: vec!["amem".to_string()],
            requirements: vec![
                "256GB+ memory".to_string(),
                "amem partition only".to_string(),
            ],
            priority: QosPriority::Normal,
            is_default: false,
        },
        QosSpec {
            id: "testing".to_string(),
            name: "testing".to_string(),
            title: "Testing & Development".to_string(),
            description: "Quick testing with limited resources".to_string(),
            max_walltime_hours: 1,
            max_jobs: 5,
            node_limit: 2,
            valid_partitions: vec![
                "atesting".to_string(),
                "atesting_a100".to_string(),
                "atesting_mi100".to_string(),
            ],
            requirements: vec![],
            priority: QosPriority::High,
            is_default: false,
        },
        QosSpec {
            id: "compile".to_string(),
            name: "compile".to_string(),
            title: "Compilation".to_string(),
            description: "For code compilation jobs".to_string(),
            max_walltime_hours: 12,
            max_jobs: 999,
            node_limit: 1,
            valid_partitions: vec!["acompile".to_string()],
            requirements: vec![],
            priority: QosPriority::Normal,
            is_default: false,
        },
    ]
}

/// Get Alpine job presets
fn get_alpine_presets() -> Vec<JobPreset> {
    vec![
        JobPreset {
            id: "small-test".to_string(),
            name: "Small Test".to_string(),
            description: "Quick test for debugging and validation".to_string(),
            icon: "🧪".to_string(),
            category: "test".to_string(),
            config: JobPresetConfig {
                cores: 24,
                memory: "16".to_string(),
                wall_time: "04:00:00".to_string(),
                partition: "amilan".to_string(),
                qos: "normal".to_string(),
            },
            estimated_cost: "96 SU".to_string(),
            estimated_queue: "< 30 min".to_string(),
            use_cases: vec![
                "Testing configurations".to_string(),
                "Small systems".to_string(),
                "Quick validation".to_string(),
            ],
            requires_gpu: false,
        },
        JobPreset {
            id: "production".to_string(),
            name: "Production Run".to_string(),
            description: "Standard production simulation".to_string(),
            icon: "⚡".to_string(),
            category: "production".to_string(),
            config: JobPresetConfig {
                cores: 48,
                memory: "32".to_string(),
                wall_time: "24:00:00".to_string(),
                partition: "amilan".to_string(),
                qos: "normal".to_string(),
            },
            estimated_cost: "1,152 SU".to_string(),
            estimated_queue: "< 2 hours".to_string(),
            use_cases: vec![
                "Standard simulations".to_string(),
                "Production runs".to_string(),
                "Most NAMD jobs".to_string(),
            ],
            requires_gpu: false,
        },
        JobPreset {
            id: "large-scale".to_string(),
            name: "Large Scale".to_string(),
            description: "High-performance parallel simulation".to_string(),
            icon: "🚀".to_string(),
            category: "large".to_string(),
            config: JobPresetConfig {
                cores: 128,
                memory: "64".to_string(),
                wall_time: "168:00:00".to_string(),
                partition: "amilan128c".to_string(),
                qos: "long".to_string(),
            },
            estimated_cost: "21,504 SU".to_string(),
            estimated_queue: "2-6 hours".to_string(),
            use_cases: vec![
                "Large systems".to_string(),
                "Long simulations".to_string(),
                "High-throughput jobs".to_string(),
            ],
            requires_gpu: false,
        },
        JobPreset {
            id: "gpu-accelerated".to_string(),
            name: "GPU Accelerated".to_string(),
            description: "Fast GPU-powered simulation".to_string(),
            icon: "🔥".to_string(),
            category: "gpu".to_string(),
            config: JobPresetConfig {
                cores: 64,
                memory: "48".to_string(),
                wall_time: "24:00:00".to_string(),
                partition: "aa100".to_string(),
                qos: "normal".to_string(),
            },
            estimated_cost: "4,107 SU".to_string(),
            estimated_queue: "1-4 hours".to_string(),
            use_cases: vec![
                "GPU-accelerated NAMD".to_string(),
                "Fast simulations".to_string(),
                "CUDA workloads".to_string(),
            ],
            requires_gpu: true,
        },
    ]
}

// ============================================================================
// Active Cluster Management
// ============================================================================

use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    static ref ACTIVE_CLUSTER: Arc<RwLock<ClusterProfile>> =
        Arc::new(RwLock::new(ClusterProfile::default()));
}

/// Get the currently active cluster profile
pub fn get_active_profile() -> ClusterProfile {
    ACTIVE_CLUSTER.read()
        .expect("Failed to read active cluster profile")
        .clone()
}

/// Set the active cluster profile (for future multi-cluster support)
pub fn set_active_profile(profile: ClusterProfile) -> Result<()> {
    validate_cluster_profile(&profile)?;

    let mut active = ACTIVE_CLUSTER.write()
        .map_err(|e| anyhow!("Failed to write active cluster profile: {}", e))?;
    *active = profile;
    Ok(())
}

// ============================================================================
// Public API (used by Tauri commands)
// ============================================================================

/// Get cluster capabilities for frontend
pub fn get_cluster_capabilities() -> ClusterCapabilities {
    get_active_profile().capabilities
}

/// Get partition limits for validation
pub fn get_partition_limits(partition_id: &str) -> Option<PartitionLimits> {
    // Partition limits for Alpine cluster
    match partition_id {
        "amilan" => Some(PartitionLimits {
            max_cores: 64,
            max_memory_per_core: 3.75,
            min_memory_for_qos: None,
        }),
        "amilan128c" => Some(PartitionLimits {
            max_cores: 128,
            max_memory_per_core: 2.01,
            min_memory_for_qos: None,
        }),
        "amem" => Some(PartitionLimits {
            max_cores: 128,
            max_memory_per_core: 21.5,
            min_memory_for_qos: Some(256),
        }),
        "aa100" | "ami100" | "al40" => Some(PartitionLimits {
            max_cores: 64,
            max_memory_per_core: 3.75,
            min_memory_for_qos: None,
        }),
        "atesting" => Some(PartitionLimits {
            max_cores: 16,
            max_memory_per_core: 4.0,
            min_memory_for_qos: None,
        }),
        "atesting_a100" => Some(PartitionLimits {
            max_cores: 10,
            max_memory_per_core: 4.0,
            min_memory_for_qos: None,
        }),
        "atesting_mi100" => Some(PartitionLimits {
            max_cores: 64,
            max_memory_per_core: 3.75,
            min_memory_for_qos: None,
        }),
        "acompile" => Some(PartitionLimits {
            max_cores: 4,
            max_memory_per_core: 4.0,
            min_memory_for_qos: None,
        }),
        _ => None,
    }
}

/// Get QOS options valid for a specific partition
pub fn get_qos_for_partition(partition_id: &str) -> Vec<QosSpec> {
    get_active_profile()
        .capabilities
        .qos_options
        .into_iter()
        .filter(|qos| qos.valid_partitions.contains(&partition_id.to_string()))
        .collect()
}

/// Calculate estimated job cost
pub fn calculate_job_cost(cores: u32, walltime_hours: f64, has_gpu: bool, gpu_count: u32) -> u32 {
    let billing = &get_active_profile().capabilities.billing_rates;
    let core_cost = cores as f64 * walltime_hours * billing.cpu_cost_per_core_hour;
    let gpu_cost = if has_gpu {
        gpu_count as f64 * walltime_hours * billing.gpu_cost_per_gpu_hour
    } else {
        0.0
    };
    (core_cost + gpu_cost).round() as u32
}

/// Estimate queue time based on resources and partition
pub fn estimate_queue_time(cores: u32, partition_id: &str) -> String {
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
pub fn suggest_qos(walltime_hours: f64, partition_id: &str) -> String {
    if partition_id == "amem" {
        "mem".to_string()
    } else if partition_id.starts_with("atesting") {
        "testing".to_string()
    } else if partition_id == "acompile" {
        "compile".to_string()
    } else if walltime_hours > 24.0 {
        let available_qos = get_qos_for_partition(partition_id);
        if available_qos.iter().any(|q| q.id == "long") {
            "long".to_string()
        } else {
            "normal".to_string()
        }
    } else {
        "normal".to_string()
    }
}

/// Get the module setup command from active cluster
pub fn get_module_setup() -> Result<String> {
    Ok(get_active_profile().connection.module_setup)
}

// ============================================================================
// Validation
// ============================================================================

/// Validate cluster profile configuration
fn validate_cluster_profile(profile: &ClusterProfile) -> Result<()> {
    // Validate cluster name
    if profile.connection.name.is_empty() || profile.connection.name.len() > 64 {
        return Err(anyhow!("Cluster name must be between 1-64 characters"));
    }
    if !profile.connection.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ') {
        return Err(anyhow!("Cluster name contains invalid characters"));
    }

    // Validate login server
    if profile.connection.login_server.is_empty() || profile.connection.login_server.len() > 253 {
        return Err(anyhow!("Login server must be a valid hostname"));
    }
    if !profile.connection.login_server.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
        return Err(anyhow!("Login server contains invalid characters"));
    }

    // Validate port (port is u16, so max is already 65535)
    if profile.connection.port == 0 {
        return Err(anyhow!("Port must be between 1-65535"));
    }

    // Validate module setup
    validate_module_setup(&profile.connection.module_setup)?;

    // Validate capabilities
    if profile.capabilities.partitions.is_empty() {
        return Err(anyhow!("Cluster must have at least one partition"));
    }
    if profile.capabilities.qos_options.is_empty() {
        return Err(anyhow!("Cluster must have at least one QoS option"));
    }

    Ok(())
}

/// Validate module setup command
fn validate_module_setup(module_setup: &str) -> Result<()> {
    // Allow empty module setup
    if module_setup.is_empty() {
        return Ok(());
    }

    // Check length
    if module_setup.len() > 1024 {
        return Err(anyhow!("Module setup command too long (max 1024 characters)"));
    }

    // Check for dangerous patterns while allowing legitimate module commands
    let dangerous_patterns = [
        "rm -rf", "dd if=", "> /etc/", "curl http", "wget http",
        "nc ", "netcat", "/dev/tcp", "exec ", "eval "
    ];

    let lower_setup = module_setup.to_lowercase();
    for pattern in &dangerous_patterns {
        if lower_setup.contains(pattern) {
            return Err(anyhow!("Module setup contains potentially dangerous command: {}", pattern));
        }
    }

    // Must contain only printable ASCII
    if !module_setup.chars().all(|c| c.is_ascii() && (c.is_ascii_graphic() || c.is_ascii_whitespace())) {
        return Err(anyhow!("Module setup contains non-printable characters"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile_is_alpine() {
        let profile = ClusterProfile::default();
        assert_eq!(profile.id, "alpine");
        assert_eq!(profile.connection.name, "Alpine");
        assert_eq!(profile.connection.login_server, "login.rc.colorado.edu");
        assert_eq!(profile.connection.port, 22);
    }

    #[test]
    fn test_alpine_has_all_partitions() {
        let profile = alpine_profile();
        let partitions = &profile.capabilities.partitions;

        assert!(partitions.iter().any(|p| p.id == "amilan"));
        assert!(partitions.iter().any(|p| p.id == "amilan128c"));
        assert!(partitions.iter().any(|p| p.id == "amem"));
        assert!(partitions.iter().any(|p| p.id == "aa100"));
        assert!(partitions.iter().any(|p| p.id == "ami100"));
        assert!(partitions.iter().any(|p| p.id == "al40"));
        assert!(partitions.iter().any(|p| p.id == "atesting"));
        assert!(partitions.iter().any(|p| p.id == "atesting_a100"));
        assert!(partitions.iter().any(|p| p.id == "atesting_mi100"));
        assert!(partitions.iter().any(|p| p.id == "acompile"));
    }

    #[test]
    fn test_alpine_has_default_partition() {
        let profile = alpine_profile();
        let default_partition = profile.capabilities.partitions.iter()
            .find(|p| p.is_default);

        assert!(default_partition.is_some());
        assert_eq!(default_partition.unwrap().id, "amilan");
    }

    #[test]
    fn test_alpine_has_all_qos() {
        let profile = alpine_profile();
        let qos = &profile.capabilities.qos_options;

        assert!(qos.iter().any(|q| q.id == "normal"));
        assert!(qos.iter().any(|q| q.id == "long"));
        assert!(qos.iter().any(|q| q.id == "mem"));
        assert!(qos.iter().any(|q| q.id == "testing"));
        assert!(qos.iter().any(|q| q.id == "compile"));
    }

    #[test]
    fn test_alpine_has_default_qos() {
        let profile = alpine_profile();
        let default_qos = profile.capabilities.qos_options.iter()
            .find(|q| q.is_default);

        assert!(default_qos.is_some());
        assert_eq!(default_qos.unwrap().id, "normal");
    }

    #[test]
    fn test_get_partition_limits() {
        assert!(get_partition_limits("amilan").is_some());
        assert!(get_partition_limits("amem").is_some());
        assert!(get_partition_limits("invalid").is_none());

        let amilan_limits = get_partition_limits("amilan").unwrap();
        assert_eq!(amilan_limits.max_cores, 64);
        assert_eq!(amilan_limits.max_memory_per_core, 3.75);
    }

    #[test]
    fn test_get_qos_for_partition() {
        let amilan_qos = get_qos_for_partition("amilan");
        assert!(amilan_qos.iter().any(|q| q.id == "normal"));
        assert!(amilan_qos.iter().any(|q| q.id == "long"));
        assert!(!amilan_qos.iter().any(|q| q.id == "mem"));

        let amem_qos = get_qos_for_partition("amem");
        assert!(amem_qos.iter().any(|q| q.id == "mem"));
        assert!(!amem_qos.iter().any(|q| q.id == "normal"));
    }

    #[test]
    fn test_calculate_job_cost() {
        // CPU only: 24 cores * 4 hours = 96 SU
        assert_eq!(calculate_job_cost(24, 4.0, false, 0), 96);

        // With GPU: (64 cores * 24 hours) + (1 GPU * 24 hours * 108.2) = 1536 + 2596.8 = 4133
        let cost = calculate_job_cost(64, 24.0, true, 1);
        assert!((4130..=4140).contains(&cost));
    }

    #[test]
    fn test_estimate_queue_time() {
        assert_eq!(estimate_queue_time(24, "amilan"), "< 30 minutes");
        assert_eq!(estimate_queue_time(48, "amilan"), "< 2 hours");
        assert_eq!(estimate_queue_time(128, "amilan128c"), "2-6 hours");
        assert_eq!(estimate_queue_time(32, "aa100"), "1-4 hours");
        assert_eq!(estimate_queue_time(8, "atesting"), "< 15 minutes");
    }

    #[test]
    fn test_suggest_qos() {
        assert_eq!(suggest_qos(12.0, "amilan"), "normal");
        assert_eq!(suggest_qos(48.0, "amilan"), "long");
        assert_eq!(suggest_qos(24.0, "amem"), "mem");
        assert_eq!(suggest_qos(0.5, "atesting"), "testing");
        assert_eq!(suggest_qos(4.0, "acompile"), "compile");
    }

    #[test]
    fn test_validate_cluster_profile() {
        let valid_profile = alpine_profile();
        assert!(validate_cluster_profile(&valid_profile).is_ok());

        // Test invalid profile
        let mut invalid = valid_profile.clone();
        invalid.connection.name = "".to_string();
        assert!(validate_cluster_profile(&invalid).is_err());
    }

    #[test]
    fn test_active_profile_management() {
        let profile = alpine_profile();
        assert!(set_active_profile(profile.clone()).is_ok());

        let active = get_active_profile();
        assert_eq!(active.id, "alpine");
    }
}
