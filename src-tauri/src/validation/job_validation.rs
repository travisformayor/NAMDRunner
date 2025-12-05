use anyhow::anyhow;
use crate::cluster::{get_partition_limits, get_qos_for_partition};
use crate::types::commands::ValidateJobConfigParams;
use crate::validation::input;
use crate::commands::helpers;

/// Business logic validation for job operations
/// Extracted from Tauri commands for independent testing
/// Validation result with detailed error information
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        ValidationResult {
            is_valid: true,
            issues: vec![],
            warnings: vec![],
            suggestions: vec![],
        }
    }

    pub fn invalid(issues: Vec<String>) -> Self {
        ValidationResult {
            is_valid: false,
            issues,
            warnings: vec![],
            suggestions: vec![],
        }
    }

    pub fn to_error(&self) -> Option<anyhow::Error> {
        if !self.is_valid {
            Some(anyhow!("Validation failed:\n{}", self.issues.join("\n")))
        } else {
            None
        }
    }
}

/// Validate resource allocation against cluster limits
pub fn validate_resource_allocation(
    config: &crate::types::core::SlurmConfig,
    partition_id: &str,
    qos_id: &str,
) -> ValidationResult {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Validate cores
    if config.cores == 0 {
        issues.push("Cores must be greater than 0".to_string());
    }

    // Parse and validate memory
    let memory_gb = match config.parse_memory_gb() {
        Ok(gb) => {
            if gb <= 0.0 {
                issues.push("Memory must be greater than 0".to_string());
            }
            gb
        }
        Err(e) => {
            issues.push(format!("Memory: {}", e));
            0.0  // Continue validation with default to collect all issues
        }
    };

    // Parse and validate walltime
    let walltime_hours = match config.parse_walltime_hours() {
        Ok(hours) => {
            if hours <= 0.0 {
                issues.push("Walltime must be greater than 0".to_string());
            }
            hours
        }
        Err(e) => {
            issues.push(format!("Walltime: {}", e));
            0.0  // Continue validation with default to collect all issues
        }
    };

    // Get partition limits
    let limits = match get_partition_limits(partition_id) {
        Some(l) => l,
        None => {
            issues.push(format!("Unknown partition: {}", partition_id));
            return ValidationResult { is_valid: false, issues, warnings, suggestions };
        }
    };

    // Validate cores against partition limit
    if config.cores > limits.max_cores {
        issues.push(format!(
            "Cores ({}) exceeds partition '{}' limit ({})",
            config.cores, partition_id, limits.max_cores
        ));
    }

    // Validate memory against partition limit
    let max_memory = config.cores as f64 * limits.max_memory_per_core;
    if memory_gb > max_memory {
        issues.push(format!(
            "Memory ({:.1}GB) exceeds limit for {} cores on partition '{}' ({:.1}GB)",
            memory_gb, config.cores, partition_id, max_memory
        ));
    }

    // Validate QOS
    let valid_qos = get_qos_for_partition(partition_id);
    if let Some(qos) = valid_qos.iter().find(|q| q.id == qos_id) {
        // Validate walltime against QOS limit
        if walltime_hours > qos.max_walltime_hours as f64 {
            issues.push(format!(
                "Walltime ({:.1}h) exceeds QOS '{}' limit ({}h)",
                walltime_hours, qos_id, qos.max_walltime_hours
            ));
        }

        // QOS-specific validation
        if qos_id == "mem" && memory_gb < 256.0 {
            issues.push("QOS 'mem' requires at least 256GB memory".to_string());
        }
    } else {
        issues.push(format!(
            "QOS '{}' is not valid for partition '{}'",
            qos_id, partition_id
        ));
    }

    // Efficiency warnings
    if config.cores < 16 {
        warnings.push("Small core count may have longer queue times".to_string());
    }

    if partition_id == "amilan128c" && config.cores < 64 {
        warnings.push("Consider 'amilan' partition for jobs under 64 cores".to_string());
    }

    if walltime_hours > 48.0 && qos_id == "normal" {
        suggestions.push("Consider 'long' QOS for runs over 48 hours".to_string());
    }

    // Memory optimization suggestions
    let recommended_memory = config.cores as f64 * 2.0; // 2GB per core is often efficient
    if memory_gb > recommended_memory * 2.0 {
        suggestions.push(format!(
            "Consider reducing memory to ~{:.0}GB for better efficiency",
            recommended_memory
        ));
    }

    ValidationResult {
        is_valid: issues.is_empty(),
        issues,
        warnings,
        suggestions,
    }
}

/// Validate complete job configuration
/// Orchestrates job name, template, and resource validation
pub async fn validate_complete_job_config(params: ValidateJobConfigParams) -> ValidationResult {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Validate job name
    if params.job_name.trim().is_empty() {
        issues.push("Job name is required".to_string());
    } else if let Err(e) = input::sanitize_job_id(&params.job_name) {
        issues.push(format!("Job name invalid: {}", e));
    }

    // Validate template selection
    if params.template_id.is_empty() {
        issues.push("Template selection is required".to_string());
    }

    // Validate template values (if template selected)
    if !params.template_id.is_empty() {
        // Load template and validate values
        match helpers::load_template_or_fail(&params.template_id, "Validation") {
            Ok(template) => {
                // Call template validation module directly (not command wrapper)
                let template_validation = crate::templates::validate_values(&template, &params.template_values);

                // Merge results
                issues.extend(template_validation.issues);
                warnings.extend(template_validation.warnings);
                suggestions.extend(template_validation.suggestions);
            }
            Err(e) => {
                issues.push(format!("Template error: {}", e));
            }
        }
    }

    // Validate resource configuration
    let slurm_config = crate::types::SlurmConfig {
        cores: params.cores,
        memory: params.memory.clone(),
        walltime: params.walltime.clone(),
        partition: params.partition.clone(),
        qos: params.qos.clone(),
    };

    let resource_validation = validate_resource_allocation(&slurm_config, &params.partition, &params.qos);

    // Merge resource validation results
    issues.extend(resource_validation.issues);
    warnings.extend(resource_validation.warnings);
    suggestions.extend(resource_validation.suggestions);

    ValidationResult {
        is_valid: issues.is_empty(),
        issues,
        warnings,
        suggestions,
    }
}

/// Validate resource allocation against cluster limits (Tauri command wrapper)
#[tauri::command(rename_all = "snake_case", rename = "validate_resource_allocation")]
pub fn validate_resource_allocation_command(
    cores: u32,
    memory: String,
    walltime: String,
    partition_id: String,
    qos_id: String,
) -> ValidationResult {
    let config = crate::types::SlurmConfig {
        cores,
        memory,
        walltime,
        partition: partition_id.clone(),
        qos: qos_id.clone(),
    };

    validate_resource_allocation(&config, &partition_id, &qos_id)
}