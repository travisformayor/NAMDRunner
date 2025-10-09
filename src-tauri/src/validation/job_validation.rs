use anyhow::{Result, anyhow};
use crate::cluster::{get_partition_limits, get_qos_for_partition};

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

/// Parse memory string to GB (e.g., "16GB", "32", "2048MB")
fn parse_memory_gb(memory: &str) -> Result<f64> {
    let clean = memory.trim().to_lowercase();

    if clean.is_empty() {
        return Err(anyhow!("Memory value is required"));
    }

    // Try to extract number and unit
    let re = regex::Regex::new(r"^(\d+(?:\.\d+)?)\s*(gb|g|mb|m)?$").unwrap();

    if let Some(captures) = re.captures(&clean) {
        let value: f64 = captures.get(1)
            .ok_or_else(|| anyhow!("Invalid memory format"))?
            .as_str()
            .parse()
            .map_err(|_| anyhow!("Invalid memory number"))?;

        let unit = captures.get(2).map(|m| m.as_str()).unwrap_or("gb");

        match unit {
            "gb" | "g" | "" => Ok(value),
            "mb" | "m" => Ok(value / 1024.0),
            _ => Err(anyhow!("Unsupported memory unit: {}", unit)),
        }
    } else {
        Err(anyhow!("Invalid memory format: {}. Use format like '16GB', '32', or '2048MB'", memory))
    }
}

/// Parse walltime string to hours (e.g., "24:00:00", "04:30:00")
fn parse_walltime_hours(walltime: &str) -> Result<f64> {
    if walltime.trim().is_empty() {
        return Err(anyhow!("Walltime is required"));
    }

    let parts: Vec<&str> = walltime.split(':').collect();

    if parts.len() != 3 {
        return Err(anyhow!("Walltime must be in HH:MM:SS format (e.g., '24:00:00')"));
    }

    let hours: u32 = parts[0].parse()
        .map_err(|_| anyhow!("Invalid hours in walltime"))?;
    let minutes: u32 = parts[1].parse()
        .map_err(|_| anyhow!("Invalid minutes in walltime"))?;
    let seconds: u32 = parts[2].parse()
        .map_err(|_| anyhow!("Invalid seconds in walltime"))?;

    if minutes >= 60 {
        return Err(anyhow!("Minutes must be less than 60"));
    }
    if seconds >= 60 {
        return Err(anyhow!("Seconds must be less than 60"));
    }

    Ok(hours as f64 + (minutes as f64 / 60.0) + (seconds as f64 / 3600.0))
}

/// Validate resource allocation against cluster limits
pub fn validate_resource_allocation(
    cores: u32,
    memory: &str,
    walltime: &str,
    partition_id: &str,
    qos_id: &str,
) -> ValidationResult {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Validate cores
    if cores == 0 {
        issues.push("Cores must be greater than 0".to_string());
    }

    // Parse memory
    let memory_gb = match parse_memory_gb(memory) {
        Ok(gb) => gb,
        Err(e) => {
            issues.push(format!("Memory: {}", e));
            return ValidationResult { is_valid: false, issues, warnings, suggestions };
        }
    };

    if memory_gb <= 0.0 {
        issues.push("Memory must be greater than 0".to_string());
    }

    // Parse walltime
    let walltime_hours = match parse_walltime_hours(walltime) {
        Ok(hours) => hours,
        Err(e) => {
            issues.push(format!("Walltime: {}", e));
            return ValidationResult { is_valid: false, issues, warnings, suggestions };
        }
    };

    if walltime_hours <= 0.0 {
        issues.push("Walltime must be greater than 0".to_string());
    }

    // Get partition limits
    let limits = match get_partition_limits(partition_id) {
        Some(l) => l,
        None => {
            issues.push(format!("Unknown partition: {}", partition_id));
            return ValidationResult { is_valid: false, issues, warnings, suggestions };
        }
    };

    // Validate cores against partition limit
    if cores > limits.max_cores {
        issues.push(format!(
            "Cores ({}) exceeds partition '{}' limit ({})",
            cores, partition_id, limits.max_cores
        ));
    }

    // Validate memory against partition limit
    let max_memory = cores as f64 * limits.max_memory_per_core;
    if memory_gb > max_memory {
        issues.push(format!(
            "Memory ({:.1}GB) exceeds limit for {} cores on partition '{}' ({:.1}GB)",
            memory_gb, cores, partition_id, max_memory
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
    if cores < 16 {
        warnings.push("Small core count may have longer queue times".to_string());
    }

    if partition_id == "amilan128c" && cores < 64 {
        warnings.push("Consider 'amilan' partition for jobs under 64 cores".to_string());
    }

    if walltime_hours > 48.0 && qos_id == "normal" {
        suggestions.push("Consider 'long' QOS for runs over 48 hours".to_string());
    }

    // Memory optimization suggestions
    let recommended_memory = cores as f64 * 2.0; // 2GB per core is often efficient
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