use crate::types::*;
use crate::database::with_database;
use crate::templates::Template;
use crate::commands::helpers;
use crate::validation::job_validation::ValidationResult;
use crate::{log_info, log_error};
use std::collections::HashMap;
use serde_json::Value;

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
            };
        }
    };

    // Use extracted validation logic (testable without database)
    let result = crate::templates::validate_values(&template, &values);

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
