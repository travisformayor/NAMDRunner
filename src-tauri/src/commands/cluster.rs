use crate::types::*;
use crate::log_debug;

/// Get cluster capabilities (partitions, QOS, job presets)
#[tauri::command(rename_all = "snake_case")]
pub async fn get_cluster_capabilities() -> ApiResult<crate::cluster::ClusterCapabilities> {
    log_debug!(category: "Cluster", message: "get_cluster_capabilities command called");

    let capabilities = crate::cluster::get_cluster_capabilities();

    log_debug!(
        category: "Cluster",
        message: "Returning cluster capabilities",
        details: "{} partitions, {} QOS options, {} presets",
        capabilities.partitions.len(),
        capabilities.qos_options.len(),
        capabilities.job_presets.len()
    );

    ApiResult {
        success: true,
        data: Some(capabilities),
        error: None,
    }
}

/// Suggest optimal QoS for given walltime and partition
#[tauri::command(rename_all = "snake_case")]
pub fn suggest_qos_for_partition(walltime_hours: f64, partition_id: String) -> String {
    crate::cluster::suggest_qos(walltime_hours, &partition_id)
}

/// Estimate queue time for given resources and partition
#[tauri::command(rename_all = "snake_case")]
pub fn estimate_queue_time_for_job(cores: u32, partition_id: String) -> String {
    crate::cluster::estimate_queue_time(cores, &partition_id)
}

/// Calculate estimated job cost in Service Units (SU)
#[tauri::command(rename_all = "snake_case")]
pub fn calculate_job_cost(cores: u32, walltime_hours: f64, has_gpu: bool, gpu_count: u32) -> u32 {
    crate::cluster::calculate_job_cost(cores, walltime_hours, has_gpu, gpu_count)
}

/// Validate resource allocation against cluster limits
#[tauri::command(rename_all = "snake_case")]
pub fn validate_resource_allocation(
    cores: u32,
    memory: String,
    walltime: String,
    partition_id: String,
    qos_id: String,
) -> crate::validation::job_validation::ValidationResult {
    let config = crate::types::core::SlurmConfig {
        cores,
        memory,
        walltime,
        partition: Some(partition_id.clone()),
        qos: Some(qos_id.clone()),
    };

    crate::validation::job_validation::validate_resource_allocation(
        &config,
        &partition_id,
        &qos_id,
    )
}
