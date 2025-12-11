/// Validation command wrappers
/// All validation-related Tauri commands that call business logic in validation/

use crate::validation::job::{ValidationResult, validate_resource_allocation};

/// Validate resource allocation against cluster limits (Tauri command wrapper)
#[tauri::command(rename_all = "snake_case")]
pub fn validate_resource_allocation_command(
    cores: u32,
    memory: String,
    walltime: String,
    partition_id: String,
    qos_id: String,
) -> ValidationResult {
    crate::log_info!(
        category: "Validation",
        message: "Validating resource allocation",
        details: "cores: {}, memory: {}, partition: {}, qos: {}", cores, memory, partition_id, qos_id
    );

    let config = crate::types::SlurmConfig {
        cores,
        memory,
        walltime,
        partition: partition_id.clone(),
        qos: qos_id.clone(),
    };

    let result = validate_resource_allocation(&config, &partition_id, &qos_id);

    crate::log_info!(
        category: "Validation",
        message: "Validation complete",
        details: "is_valid: {}, issues: {}", result.is_valid, result.issues.len()
    );

    result
}
