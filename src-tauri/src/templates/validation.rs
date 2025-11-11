use super::{Template, VariableType};
use serde_json::Value;
use std::collections::HashMap;
use crate::validation::job_validation::ValidationResult;

/// Validate template values against template variable definitions
/// Pure business logic - no database or AppHandle dependencies
pub fn validate_values(template: &Template, values: &HashMap<String, Value>) -> ValidationResult {
    let mut issues = Vec::new();

    // Check all template variables have values (all variables are required)
    for (key, var_def) in &template.variables {
        if !values.contains_key(key) {
            issues.push(format!("Required variable missing: {}", var_def.label));
        }
    }

    // Validate value types and ranges
    for (key, value) in values {
        if let Some(var_def) = template.variables.get(key) {
            match &var_def.var_type {
                VariableType::Number { min, max, .. } => {
                    if let Some(num) = value.as_f64() {
                        if num < *min {
                            issues.push(format!("{}: value {} below minimum {}", var_def.label, num, min));
                        }
                        if num > *max {
                            issues.push(format!("{}: value {} above maximum {}", var_def.label, num, max));
                        }
                    } else {
                        issues.push(format!("{}: expected number, got {:?}", var_def.label, value));
                    }
                }
                VariableType::Text { .. } => {
                    if !value.is_string() {
                        issues.push(format!("{}: expected text, got {:?}", var_def.label, value));
                    }
                }
                VariableType::Boolean { .. } => {
                    if !value.is_boolean() {
                        issues.push(format!("{}: expected boolean, got {:?}", var_def.label, value));
                    }
                }
                VariableType::FileUpload { extensions } => {
                    if let Some(filename) = value.as_str() {
                        // Check file is provided (not empty)
                        if filename.trim().is_empty() {
                            issues.push(format!("{}: file is required", var_def.label));
                        } else {
                            // Validate file extension
                            let ext_match = extensions.iter().any(|ext| filename.to_lowercase().ends_with(&ext.to_lowercase()));
                            if !ext_match {
                                issues.push(format!("{}: file '{}' does not match allowed extensions: {:?}", var_def.label, filename, extensions));
                            }
                        }
                    } else {
                        issues.push(format!("{}: expected filename string, got {:?}", var_def.label, value));
                    }
                }
            }
        }
    }

    ValidationResult {
        is_valid: issues.is_empty(),
        issues,
        warnings: vec![],
        suggestions: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::VariableDefinition;

    fn create_test_template() -> Template {
        let mut variables = HashMap::new();

        variables.insert(
            "temperature".to_string(),
            VariableDefinition {
                key: "temperature".to_string(),
                label: "Temperature (K)".to_string(),
                var_type: VariableType::Number {
                    min: 200.0,
                    max: 400.0,
                    default: 300.0,
                },
                help_text: None,
            },
        );

        variables.insert(
            "structure_file".to_string(),
            VariableDefinition {
                key: "structure_file".to_string(),
                label: "Structure File".to_string(),
                var_type: VariableType::FileUpload {
                    extensions: vec![".psf".to_string(), ".pdb".to_string()],
                },
                help_text: None,
            },
        );

        variables.insert(
            "pme_enabled".to_string(),
            VariableDefinition {
                key: "pme_enabled".to_string(),
                label: "PME Enabled".to_string(),
                var_type: VariableType::Boolean { default: true },
                help_text: None,
            },
        );

        Template {
            id: "test_template".to_string(),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            namd_config_template: "test".to_string(),
            variables,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            is_builtin: false,
        }
    }

    #[test]
    fn test_valid_values_pass() {
        let template = create_test_template();
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from(300.0));
        values.insert("structure_file".to_string(), Value::from("structure.psf"));
        values.insert("pme_enabled".to_string(), Value::from(true));

        let result = validate_values(&template, &values);
        assert!(result.is_valid, "Valid values should pass validation");
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_missing_required_variable() {
        let template = create_test_template();
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from(300.0));
        // Missing structure_file and pme_enabled

        let result = validate_values(&template, &values);
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 2, "Should report 2 missing required variables");
        assert!(result.issues.iter().any(|e| e.contains("Structure File")));
        assert!(result.issues.iter().any(|e| e.contains("PME Enabled")));
    }

    #[test]
    fn test_number_below_minimum() {
        let template = create_test_template();
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from(150.0)); // Below min 200
        values.insert("structure_file".to_string(), Value::from("structure.psf"));
        values.insert("pme_enabled".to_string(), Value::from(true));

        let result = validate_values(&template, &values);
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].contains("below minimum"));
        assert!(result.issues[0].contains("150"));
        assert!(result.issues[0].contains("200"));
    }

    #[test]
    fn test_number_above_maximum() {
        let template = create_test_template();
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from(500.0)); // Above max 400
        values.insert("structure_file".to_string(), Value::from("structure.psf"));
        values.insert("pme_enabled".to_string(), Value::from(true));

        let result = validate_values(&template, &values);
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].contains("above maximum"));
        assert!(result.issues[0].contains("500"));
        assert!(result.issues[0].contains("400"));
    }

    #[test]
    fn test_wrong_type_number() {
        let template = create_test_template();
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from("not a number"));
        values.insert("structure_file".to_string(), Value::from("structure.psf"));
        values.insert("pme_enabled".to_string(), Value::from(true));

        let result = validate_values(&template, &values);
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].contains("expected number"));
    }

    #[test]
    fn test_wrong_type_boolean() {
        let template = create_test_template();
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from(300.0));
        values.insert("structure_file".to_string(), Value::from("structure.psf"));
        values.insert("pme_enabled".to_string(), Value::from("yes")); // String not bool

        let result = validate_values(&template, &values);
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].contains("expected boolean"));
    }

    #[test]
    fn test_file_extension_validation() {
        let template = create_test_template();
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from(300.0));
        values.insert("structure_file".to_string(), Value::from("structure.xyz")); // Wrong extension
        values.insert("pme_enabled".to_string(), Value::from(true));

        let result = validate_values(&template, &values);
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].contains("does not match allowed extensions"));
        assert!(result.issues[0].contains(".psf"));
    }

    #[test]
    fn test_file_extension_case_insensitive() {
        let template = create_test_template();
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from(300.0));
        values.insert("structure_file".to_string(), Value::from("structure.PSF")); // Uppercase
        values.insert("pme_enabled".to_string(), Value::from(true));

        let result = validate_values(&template, &values);
        assert!(result.is_valid, "File extension matching should be case-insensitive");
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_multiple_validation_errors() {
        let template = create_test_template();
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from(500.0)); // Above max
        values.insert("pme_enabled".to_string(), Value::from("invalid")); // Wrong type
        // Missing structure_file

        let result = validate_values(&template, &values);
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 3, "Should report all validation errors");
    }
}
