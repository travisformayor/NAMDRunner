use crate::types::*;
use crate::database::with_database;
use crate::templates::Template;
use crate::commands::helpers;
use crate::validation::job::ValidationResult;
use crate::{log_info, log_error};
use std::collections::HashMap;
use serde_json::Value;
use anyhow::{Result, anyhow};

/// List all available templates (returns summary info)
#[tauri::command(rename_all = "snake_case")]
pub async fn list_templates() -> ApiResult<Vec<crate::templates::TemplateSummary>> {
    log_info!(category: "Templates", message: "Listing all templates from database");

    // Load default templates if this is the first call and DB is empty
    // This ensures defaults are available and logging works (frontend is connected)
    if let Err(e) = crate::database::ensure_default_templates_loaded() {
        log_error!(category: "Templates", message: "Failed to ensure default templates", details: "Error: {}", e);
    }

    match with_database(|db| db.list_templates()) {
        Ok(templates) => {
            log_info!(category: "Templates", message: "Templates loaded", details: "Found {} templates", templates.len());
            ApiResult::success(templates)
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Failed to list templates", details: "Database error: {}", e);
            ApiResult::error(format!("Database error: {}", e))
        }
    }
}

/// Get full template definition by ID
#[tauri::command(rename_all = "snake_case")]
pub async fn get_template(template_id: String) -> ApiResult<Template> {
    log_info!(category: "Templates", message: "Loading template", details: "ID: {}", template_id);

    let template = match helpers::load_template_or_fail(&template_id, "Templates") {
        Ok(t) => t,
        Err(e) => return ApiResult::error(e.to_string()),
    };

    log_info!(category: "Templates", message: "Template loaded", details: "Name: {}", template.name);
    ApiResult::success(template)
}

/// Create a new template
#[tauri::command(rename_all = "snake_case")]
pub async fn create_template(template: Template) -> ApiResult<String> {
    log_info!(category: "Templates", message: "Creating template", details: "ID: {}", template.id);

    // Check if template ID already exists
    match with_database(|db| db.load_template(&template.id)) {
        Ok(Some(_)) => {
            log_error!(category: "Templates", message: "Template ID already exists", details: "ID: {}", template.id);
            return ApiResult::error(format!("Template with ID '{}' already exists", template.id));
        }
        Ok(None) => {
            // ID is available, proceed
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Database error checking template", details: "Error: {}", e);
            return ApiResult::error(format!("Database error: {}", e));
        }
    }

    // Save the new template
    let template_id = template.id.clone();
    match with_database(|db| db.save_template(&template)) {
        Ok(_) => {
            log_info!(category: "Templates", message: "Template created successfully", show_toast: true);
            ApiResult::success(template_id)
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Failed to save template", details: "Error: {}", e);
            ApiResult::error(format!("Failed to save template: {}", e))
        }
    }
}

/// Update an existing template
#[tauri::command(rename_all = "snake_case")]
pub async fn update_template(template_id: String, template: Template) -> ApiResult<()> {
    log_info!(category: "Templates", message: "Updating template", details: "ID: {}", template_id);

    // Verify template exists
    if let Err(e) = helpers::load_template_or_fail(&template_id, "Templates") {
        return ApiResult::error(e.to_string());
    }

    // Save updated template
    match with_database(|db| db.save_template(&template)) {
        Ok(_) => {
            log_info!(category: "Templates", message: "Template updated successfully", show_toast: true);
            ApiResult::success(())
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Failed to update template", details: "Error: {}", e);
            ApiResult::error(format!("Failed to update template: {}", e))
        }
    }
}

/// Delete a template (blocked if jobs are using it)
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_template(template_id: String) -> ApiResult<()> {
    log_info!(category: "Templates", message: "Deleting template", details: "ID: {}", template_id);

    // Check if any jobs are using this template
    let job_count = match with_database(|db| db.count_jobs_using_template(&template_id)) {
        Ok(count) => count,
        Err(e) => {
            log_error!(category: "Templates", message: "Failed to count jobs using template", details: "Error: {}", e);
            return ApiResult::error(format!("Database error: {}", e));
        }
    };

    if job_count > 0 {
        log_error!(category: "Templates", message: "Cannot delete template - jobs are using it", details: "Template: {}, Jobs: {}", template_id, job_count);
        return ApiResult::error(format!("Cannot delete template: {} job(s) are using it", job_count));
    }

    match with_database(|db| db.delete_template(&template_id)) {
        Ok(true) => {
            log_info!(category: "Templates", message: "Template deleted successfully", show_toast: true);
            ApiResult::success(())
        }
        Ok(false) => {
            log_error!(category: "Templates", message: "Template not found", details: "ID: {}", template_id);
            ApiResult::error(format!("Template '{}' not found", template_id))
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Database error deleting template", details: "Error: {}", e);
            ApiResult::error(format!("Database error: {}", e))
        }
    }
}

/// Validate template values against template definition
#[tauri::command(rename_all = "snake_case")]
pub async fn validate_template_values(
    template_id: String,
    values: HashMap<String, Value>,
) -> ValidationResult {

    // Load template
    let template = match helpers::load_template_or_fail(&template_id, "Templates") {
        Ok(t) => t,
        Err(e) => {
            return ValidationResult {
                is_valid: false,
                issues: vec![e.to_string()],
                warnings: vec![],
                suggestions: vec![],
                field_errors: None,
            };
        }
    };

    // Use extracted validation logic (testable without database)
    let result = crate::validation::template::validate_values(&template, &values);

    if !result.is_valid {
        log_error!(category: "Templates", message: "Template validation failed", details: "{} issues found", result.issues.len());
    }

    result
}

/// Preview NAMD config with user values
#[tauri::command(rename_all = "snake_case")]
pub async fn preview_namd_config(
    template_id: String,
    values: HashMap<String, Value>,
) -> ApiResult<String> {
    log_info!(category: "Templates", message: "Previewing NAMD config", details: "Template: {}", template_id);

    // Load template
    let template = match helpers::load_template_or_fail(&template_id, "Templates") {
        Ok(t) => t,
        Err(e) => return ApiResult::error(e.to_string()),
    };

    // Render template
    match crate::templates::render_template(&template, &values) {
        Ok(rendered) => {
            log_info!(category: "Templates", message: "Preview generated successfully");
            ApiResult::success(rendered)
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Preview render failed", details: "Error: {}", e);
            ApiResult::error(format!("Rendering error: {}", e))
        }
    }
}

/// Preview template with default/sample values (for template editor testing)
#[tauri::command(rename_all = "snake_case")]
pub async fn preview_template_with_defaults(template_id: String) -> ApiResult<String> {
    log_info!(category: "Templates", message: "Previewing template with defaults", details: "Template: {}", template_id);

    // Load template
    let template = match helpers::load_template_or_fail(&template_id, "Templates") {
        Ok(t) => t,
        Err(e) => return ApiResult::error(e.to_string()),
    };

    // Generate sample values from variable defaults
    let mut values = HashMap::new();
    for (key, var_def) in &template.variables {
        let sample_value = match &var_def.var_type {
            crate::templates::VariableType::Number { default, .. } => Value::from(*default),
            crate::templates::VariableType::Text { default } => Value::from(default.clone()),
            crate::templates::VariableType::Boolean { default } => Value::from(*default),
            crate::templates::VariableType::FileUpload { extensions } => {
                // Generate sample filename (renderer will prepend input_files/)
                let default_ext = ".dat".to_string();
                let ext = extensions.first().unwrap_or(&default_ext);
                Value::from(format!("{}{}", key, ext))
            }
        };
        values.insert(key.clone(), sample_value);
    }

    // Use the same renderer as preview_namd_config
    match crate::templates::render_template(&template, &values) {
        Ok(rendered) => {
            log_info!(category: "Templates", message: "Preview with defaults generated successfully");
            ApiResult::success(rendered)
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Preview with defaults failed", details: "Error: {}", e);
            ApiResult::error(format!("Rendering error: {}", e))
        }
    }
}

/// Export a template to a JSON file
#[tauri::command(rename_all = "snake_case")]
pub async fn export_template(template_id: String) -> ApiResult<String> {
    log_info!(category: "Templates", message: "Exporting template", details: "ID: {}", template_id);

    // Load template from database
    let template = match helpers::load_template_or_fail(&template_id, "Templates") {
        Ok(t) => t,
        Err(e) => return ApiResult::error(e.to_string()),
    };

    // Sanitize template name for filename
    let safe_name = sanitize_filename(&template.name);
    let default_filename = format!("{}.json", safe_name);

    // Show save dialog
    use rfd::FileDialog;
    let save_path = FileDialog::new()
        .set_file_name(&default_filename)
        .set_title("Export Template")
        .add_filter("JSON Template", &["json"])
        .save_file();

    let path = match save_path {
        Some(p) => p,
        None => {
            log_info!(category: "Templates", message: "Export cancelled by user");
            return ApiResult::error("Export cancelled".to_string());
        }
    };

    // Serialize template to pretty JSON
    let json_content = match serde_json::to_string_pretty(&template) {
        Ok(json) => json,
        Err(e) => {
            log_error!(category: "Templates", message: "Failed to serialize template", details: "Error: {}", e);
            return ApiResult::error(format!("Serialization error: {}", e));
        }
    };

    // Write to file
    match std::fs::write(&path, json_content) {
        Ok(_) => {
            let path_str = path.to_string_lossy().to_string();
            log_info!(category: "Templates", message: "Template exported successfully", details: "{}", path_str, show_toast: true);
            ApiResult::success(path_str)
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Failed to write template file", details: "Error: {}", e);
            ApiResult::error(format!("Failed to write file: {}", e))
        }
    }
}

/// Import a template from a JSON file
#[tauri::command(rename_all = "snake_case")]
pub async fn import_template() -> ApiResult<Template> {
    log_info!(category: "Templates", message: "Starting template import");

    // Show open dialog
    use rfd::FileDialog;
    let source_path = FileDialog::new()
        .set_title("Import Template")
        .add_filter("JSON Template", &["json"])
        .pick_file();

    let path = match source_path {
        Some(p) => p,
        None => {
            log_info!(category: "Templates", message: "Import cancelled by user");
            return ApiResult::error("Import cancelled".to_string());
        }
    };

    // Read file
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            log_error!(category: "Templates", message: "Failed to read template file", details: "Error: {}", e);
            return ApiResult::error(format!("Failed to read file: {}", e));
        }
    };

    // Deserialize JSON to Template
    let mut template: Template = match serde_json::from_str(&content) {
        Ok(t) => t,
        Err(e) => {
            log_error!(category: "Templates", message: "Invalid template JSON", details: "Error: {}", e);
            return ApiResult::error(format!("Invalid template file: {}", e));
        }
    };

    // Validate template structure
    if let Err(e) = validate_template_structure(&template) {
        log_error!(category: "Templates", message: "Template validation failed", details: "Error: {}", e);
        return ApiResult::error(e.to_string());
    }

    // Handle ID conflicts - generate new ID if needed
    match with_database(|db| db.load_template(&template.id)) {
        Ok(Some(_existing)) => {
            // Template with this ID exists - generate new ID
            let timestamp = chrono::Utc::now().timestamp();
            let old_id = template.id.clone();
            template.id = format!("{}_imported_{}", old_id, timestamp);
            log_info!(category: "Templates", message: "Template ID conflict resolved", details: "Original: '{}', New: '{}'", old_id, template.id);
        }
        Ok(None) => {
            // ID is available
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Database error checking template ID", details: "Error: {}", e);
            return ApiResult::error(format!("Database error: {}", e));
        }
    }

    // Save imported template
    let template_id = template.id.clone();
    let template_name = template.name.clone();
    match with_database(|db| db.save_template(&template)) {
        Ok(_) => {
            log_info!(category: "Templates", message: "Template imported successfully", details: "Name: '{}', ID: '{}'", template_name, template_id, show_toast: true);
            ApiResult::success(template)
        }
        Err(e) => {
            log_error!(category: "Templates", message: "Failed to save imported template", details: "Error: {}", e);
            ApiResult::error(format!("Failed to save template: {}", e))
        }
    }
}

/// Sanitize a string for use as a filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                // Replace whitespace and any other non-alphanumeric characters with underscore
                '_'
            }
        })
        .collect()
}

/// Validate template structure for import
fn validate_template_structure(template: &Template) -> Result<()> {
    // Validate required fields
    if template.id.trim().is_empty() {
        return Err(anyhow!("Template ID is required"));
    }

    if template.name.trim().is_empty() {
        return Err(anyhow!("Template name is required"));
    }

    if template.namd_config_template.trim().is_empty() {
        return Err(anyhow!("NAMD config template is required"));
    }

    // Validate ID contains only safe characters
    if !template.id.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(anyhow!("Template ID must contain only alphanumeric characters and underscores"));
    }

    // Validate variables
    if template.variables.is_empty() {
        return Err(anyhow!("Template must define at least one variable"));
    }

    for (key, var_def) in &template.variables {
        // Validate variable key
        if key.trim().is_empty() {
            return Err(anyhow!("Variable key cannot be empty"));
        }

        if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(anyhow!("Variable key '{}' must contain only alphanumeric characters and underscores", key));
        }

        // Validate variable label
        if var_def.label.trim().is_empty() {
            return Err(anyhow!("Variable '{}' must have a label", key));
        }

        // Validate variable type constraints
        match &var_def.var_type {
            crate::templates::VariableType::Number { min, max, default } => {
                if min > max {
                    return Err(anyhow!("Variable '{}': min ({}) cannot be greater than max ({})", key, min, max));
                }
                if default < min || default > max {
                    return Err(anyhow!("Variable '{}': default ({}) must be between min ({}) and max ({})", key, default, min, max));
                }
            }
            crate::templates::VariableType::FileUpload { extensions } => {
                if extensions.is_empty() {
                    return Err(anyhow!("Variable '{}': FileUpload must specify at least one extension", key));
                }
                for ext in extensions {
                    if !ext.starts_with('.') {
                        return Err(anyhow!("Variable '{}': Extension '{}' must start with a dot", key, ext));
                    }
                }
            }
            _ => {} // Text and Boolean have no constraints to validate
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::{VariableDefinition, VariableType};

    fn create_test_template(id: &str, name: &str) -> Template {
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
                help_text: Some("Simulation temperature".to_string()),
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
                help_text: Some("PSF structure file".to_string()),
            },
        );

        Template {
            id: id.to_string(),
            name: name.to_string(),
            description: "Test template".to_string(),
            namd_config_template: "temperature {{temperature}}\nstructure {{structure_file}}".to_string(),
            variables,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Valid Name 123"), "Valid_Name_123");
        assert_eq!(sanitize_filename("test@#$%template"), "test____template");
        assert_eq!(sanitize_filename("my-template_v1"), "my-template_v1");
        assert_eq!(sanitize_filename("  spaces  "), "__spaces__");
    }

    #[test]
    fn test_validate_template_structure_valid() {
        let template = create_test_template("test_template", "Test Template");
        let result = validate_template_structure(&template);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_template_structure_empty_id() {
        let template = create_test_template("", "Test Template");
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ID is required"));
    }

    #[test]
    fn test_validate_template_structure_empty_name() {
        let template = create_test_template("test_id", "");
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name is required"));
    }

    #[test]
    fn test_validate_template_structure_invalid_id_characters() {
        let template = create_test_template("test-template!", "Test");
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("alphanumeric"));
    }

    #[test]
    fn test_validate_template_structure_empty_config() {
        let mut template = create_test_template("test_id", "Test");
        template.namd_config_template = "".to_string();
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("config template is required"));
    }

    #[test]
    fn test_validate_template_structure_no_variables() {
        let mut template = create_test_template("test_id", "Test");
        template.variables.clear();
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one variable"));
    }

    #[test]
    fn test_validate_template_structure_empty_variable_key() {
        let mut template = create_test_template("test_id", "Test");
        template.variables.insert(
            "".to_string(),
            VariableDefinition {
                key: "".to_string(),
                label: "Label".to_string(),
                var_type: VariableType::Text {
                    default: "test".to_string(),
                },
                help_text: None,
            },
        );
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("key cannot be empty"));
    }

    #[test]
    fn test_validate_template_structure_invalid_variable_key() {
        let mut template = create_test_template("test_id", "Test");
        template.variables.insert(
            "bad-key!".to_string(),
            VariableDefinition {
                key: "bad-key!".to_string(),
                label: "Label".to_string(),
                var_type: VariableType::Text {
                    default: "test".to_string(),
                },
                help_text: None,
            },
        );
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("alphanumeric"));
    }

    #[test]
    fn test_validate_template_structure_invalid_number_range() {
        let mut template = create_test_template("test_id", "Test");
        template.variables.insert(
            "bad_range".to_string(),
            VariableDefinition {
                key: "bad_range".to_string(),
                label: "Bad Range".to_string(),
                var_type: VariableType::Number {
                    min: 100.0,
                    max: 50.0, // max < min
                    default: 75.0,
                },
                help_text: None,
            },
        );
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("min"));
        assert!(error.contains("max"));
    }

    #[test]
    fn test_validate_template_structure_default_out_of_range() {
        let mut template = create_test_template("test_id", "Test");
        template.variables.insert(
            "out_of_range".to_string(),
            VariableDefinition {
                key: "out_of_range".to_string(),
                label: "Out of Range".to_string(),
                var_type: VariableType::Number {
                    min: 0.0,
                    max: 100.0,
                    default: 200.0, // out of range
                },
                help_text: None,
            },
        );
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("default"));
        assert!(error.contains("between"));
    }

    #[test]
    fn test_validate_template_structure_file_upload_no_extensions() {
        let mut template = create_test_template("test_id", "Test");
        template.variables.insert(
            "file".to_string(),
            VariableDefinition {
                key: "file".to_string(),
                label: "File".to_string(),
                var_type: VariableType::FileUpload {
                    extensions: vec![],
                },
                help_text: None,
            },
        );
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one extension"));
    }

    #[test]
    fn test_validate_template_structure_file_upload_invalid_extension() {
        let mut template = create_test_template("test_id", "Test");
        template.variables.insert(
            "file".to_string(),
            VariableDefinition {
                key: "file".to_string(),
                label: "File".to_string(),
                var_type: VariableType::FileUpload {
                    extensions: vec!["psf".to_string()], // missing dot
                },
                help_text: None,
            },
        );
        let result = validate_template_structure(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with a dot"));
    }

    #[test]
    fn test_template_json_round_trip() {
        let template = create_test_template("test_template", "Test Template");

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&template).unwrap();

        // Deserialize back
        let deserialized: Template = serde_json::from_str(&json).unwrap();

        // Verify round-trip
        assert_eq!(template.id, deserialized.id);
        assert_eq!(template.name, deserialized.name);
        assert_eq!(template.description, deserialized.description);
        assert_eq!(template.namd_config_template, deserialized.namd_config_template);
        assert_eq!(template.variables.len(), deserialized.variables.len());
    }

    #[test]
    fn test_template_json_format() {
        let template = create_test_template("test_template_v1", "Test Template");

        let json = serde_json::to_string_pretty(&template).unwrap();

        // Verify JSON has expected top-level keys
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"description\""));
        assert!(json.contains("\"namd_config_template\""));
        assert!(json.contains("\"variables\""));
        assert!(json.contains("\"created_at\""));
        assert!(json.contains("\"updated_at\""));
    }
}
