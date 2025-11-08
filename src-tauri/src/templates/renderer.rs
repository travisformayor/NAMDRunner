use crate::templates::{Template, VariableType};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use serde_json::Value;

/// Render a template by substituting {{variables}} with actual values
/// File paths are extracted to filenames and get "input_files/" prepended automatically
pub fn render_template(
    template: &Template,
    values: &HashMap<String, Value>,
) -> Result<String> {
    let mut rendered = template.namd_config_template.clone();

    // Replace each variable with its value
    for (key, var_def) in &template.variables {
        // Get the value for this variable
        let value = values.get(key)
            .ok_or_else(|| anyhow!("Missing required variable: {}", key))?;

        // Convert value to string based on variable type
        let value_str = match &var_def.var_type {
            VariableType::FileUpload { .. } => {
                // File uploads: extract filename from potential full path, then prepend input_files/
                let file_path = value.as_str()
                    .ok_or_else(|| anyhow!("Variable {} must be a string (filename)", key))?;

                // Extract filename from full path if present (handles both "/home/user/file.psf" and "file.psf")
                let filename = std::path::Path::new(file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(file_path);  // Fallback to original if extraction fails

                format!("{}/{}", crate::ssh::JobDirectoryStructure::INPUT_FILES, filename)
            }
            VariableType::Number { .. } => {
                // Numbers: convert to string
                if let Some(num) = value.as_f64() {
                    // Format number appropriately (remove unnecessary decimals for integers)
                    if num.fract() == 0.0 {
                        format!("{:.0}", num)
                    } else {
                        format!("{}", num)
                    }
                } else if let Some(num) = value.as_i64() {
                    format!("{}", num)
                } else {
                    return Err(anyhow!("Variable {} must be a number", key));
                }
            }
            VariableType::Boolean { .. } => {
                // Booleans: convert to "yes"/"no" for NAMD
                let bool_val = value.as_bool()
                    .ok_or_else(|| anyhow!("Variable {} must be a boolean", key))?;
                if bool_val { "yes" } else { "no" }.to_string()
            }
            VariableType::Text { .. } => {
                // Text: use as-is
                value.as_str()
                    .ok_or_else(|| anyhow!("Variable {} must be a string", key))?
                    .to_string()
            }
        };

        // Replace all occurrences of {{key}} with value_str
        let placeholder = format!("{{{{{}}}}}", key);
        rendered = rendered.replace(&placeholder, &value_str);
    }

    // Check for any unreplaced variables (indicates missing values or typo in template)
    if rendered.contains("{{") && rendered.contains("}}") {
        // Find unreplaced variables for error message
        let unreplaced: Vec<&str> = rendered
            .split("{{")
            .skip(1)
            .filter_map(|s| s.split("}}").next())
            .collect();

        if !unreplaced.is_empty() {
            return Err(anyhow!(
                "Template contains unreplaced variables: {}",
                unreplaced.join(", ")
            ));
        }
    }

    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::VariableDefinition;

    #[test]
    fn test_render_simple_template() {
        let mut variables = HashMap::new();
        variables.insert(
            "temperature".to_string(),
            VariableDefinition {
                key: "temperature".to_string(),
                label: "Temperature".to_string(),
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
                    extensions: vec![".psf".to_string()],
                },
                help_text: None,
            },
        );

        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test template".to_string(),
            namd_config_template: "temperature {{temperature}}\nstructure {{structure_file}}".to_string(),
            variables,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            is_builtin: false,
        };

        let mut values = HashMap::new();
        values.insert("temperature".to_string(), Value::from(310.5));
        values.insert("structure_file".to_string(), Value::from("structure.psf"));

        let rendered = render_template(&template, &values).unwrap();
        assert_eq!(rendered, "temperature 310.5\nstructure input_files/structure.psf");
    }

    #[test]
    fn test_render_boolean_to_yes_no() {
        let mut variables = HashMap::new();
        variables.insert(
            "pme_enabled".to_string(),
            VariableDefinition {
                key: "pme_enabled".to_string(),
                label: "PME".to_string(),
                var_type: VariableType::Boolean { default: true },
                help_text: None,
            },
        );

        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test template".to_string(),
            namd_config_template: "PME {{pme_enabled}}".to_string(),
            variables,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            is_builtin: false,
        };

        let mut values_true = HashMap::new();
        values_true.insert("pme_enabled".to_string(), Value::from(true));
        let rendered_true = render_template(&template, &values_true).unwrap();
        assert_eq!(rendered_true, "PME yes");

        let mut values_false = HashMap::new();
        values_false.insert("pme_enabled".to_string(), Value::from(false));
        let rendered_false = render_template(&template, &values_false).unwrap();
        assert_eq!(rendered_false, "PME no");
    }

    #[test]
    fn test_missing_variable_error() {
        let mut variables = HashMap::new();
        variables.insert(
            "temperature".to_string(),
            VariableDefinition {
                key: "temperature".to_string(),
                label: "Temperature".to_string(),
                var_type: VariableType::Number {
                    min: 0.0,
                    max: 1000.0,
                    default: 300.0,
                },
                help_text: None,
            },
        );

        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test template".to_string(),
            namd_config_template: "temperature {{temperature}}".to_string(),
            variables,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            is_builtin: false,
        };

        let values = HashMap::new(); // Empty - missing temperature

        let result = render_template(&template, &values);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing required variable: temperature"));
    }

    #[test]
    fn test_unreplaced_variable_error() {
        let variables = HashMap::new(); // No variables defined

        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test template".to_string(),
            namd_config_template: "temperature {{undefined_var}}".to_string(),
            variables,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            is_builtin: false,
        };

        let values = HashMap::new();

        let result = render_template(&template, &values);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unreplaced variables"));
    }

    #[test]
    fn test_integer_formatting() {
        let mut variables = HashMap::new();
        variables.insert(
            "steps".to_string(),
            VariableDefinition {
                key: "steps".to_string(),
                label: "Steps".to_string(),
                var_type: VariableType::Number {
                    min: 1.0,
                    max: 100000000.0,
                    default: 1000000.0,
                },
                help_text: None,
            },
        );

        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test template".to_string(),
            namd_config_template: "run {{steps}}".to_string(),
            variables,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            is_builtin: false,
        };

        let mut values = HashMap::new();
        values.insert("steps".to_string(), Value::from(10000));

        let rendered = render_template(&template, &values).unwrap();
        assert_eq!(rendered, "run 10000"); // Should not have .0
    }
}
