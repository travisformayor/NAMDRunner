/// Cluster configuration command wrappers
/// All cluster-related Tauri commands that call business logic in cluster.rs

use crate::cluster;
use crate::types::ApiResult;

/// Get cluster capabilities for frontend
#[tauri::command(rename_all = "snake_case")]
pub fn get_cluster_capabilities() -> cluster::ClusterCapabilities {
    cluster::get_cluster_capabilities()
}

/// Save cluster config to database and update cache
#[tauri::command(rename_all = "snake_case")]
pub fn save_cluster_config(config: cluster::ClusterCapabilities) -> ApiResult<()> {
    cluster::save_cluster_config(config)
}

/// Reset cluster config to defaults (clears DB, re-seeds from embedded JSON)
#[tauri::command(rename_all = "snake_case")]
pub fn reset_cluster_config() -> ApiResult<cluster::ClusterCapabilities> {
    cluster::reset_cluster_config()
}

/// Calculate estimated job cost
#[tauri::command(rename_all = "snake_case")]
pub fn calculate_job_cost(cores: u32, walltime: String, has_gpu: bool, gpu_count: u32) -> u32 {
    cluster::calculate_job_cost(cores, walltime, has_gpu, gpu_count)
}

/// Estimate queue time based on resources and partition
#[tauri::command(rename_all = "snake_case", rename = "estimate_queue_time_for_job")]
pub fn estimate_queue_time(cores: u32, partition_id: String) -> String {
    cluster::estimate_queue_time(cores, partition_id)
}

/// Suggest optimal QOS based on walltime and partition
#[tauri::command(rename_all = "snake_case")]
pub fn suggest_qos(walltime_hours: f64, partition_id: String) -> String {
    cluster::suggest_qos(walltime_hours, partition_id)
}
