use crate::types::*;
use crate::database::with_database;
use crate::templates::Template;
use crate::validation::job_validation::ValidationResult;
use crate::{info_log, error_log};
use std::collections::HashMap;
use serde_json::Value;

/// List all available templates (returns summary info)
#[tauri::command(rename_all = "snake_case")]
pub async fn list_templates() -> ApiResult<Vec<crate::templates::TemplateSummary>> {
    info_log!("[Templates] Listing all templates from database");

    // Load default templates if this is the first call and DB is empty
    // This ensures defaults are available and logging works (frontend is connected)
    if let Err(e) = crate::database::ensure_default_templates_loaded() {
        error_log!("[Templates] Failed to ensure default templates: {}", e);
    }

    match with_database(|db| db.list_templates()) {
        Ok(templates) => {
            info_log!("[Templates] Found {} templates in DB", templates.len());
            ApiResult::success(templates)
        }
        Err(e) => {
            error_log!("[Templates] Failed to list templates: {}", e);
            ApiResult::error(format!("Database error: {}", e))
        }
    }
}

/// Get full template definition by ID
#[tauri::command(rename_all = "snake_case")]
pub async fn get_template(template_id: String) -> ApiResult<Template> {
    info_log!("[Templates] Loading template: {}", template_id);

    match with_database(|db| db.load_template(&template_id)) {
        Ok(Some(template)) => {
            info_log!("[Templates] Loaded template: {}", template.name);
            ApiResult::success(template)
        }
        Ok(None) => {
            error_log!("[Templates] Template not found: {}", template_id);
            ApiResult::error(format!("Template '{}' not found", template_id))
        }
        Err(e) => {
            error_log!("[Templates] Database error: {}", e);
            ApiResult::error(format!("Database error: {}", e))
        }
    }
}

/// Create a new template
#[tauri::command(rename_all = "snake_case")]
pub async fn create_template(template: Template) -> ApiResult<String> {
    info_log!("[Templates] Creating template: {}", template.id);

    // Check if template ID already exists
    match with_database(|db| db.load_template(&template.id)) {
        Ok(Some(_)) => {
            error_log!("[Templates] Template ID already exists: {}", template.id);
            return ApiResult::error(format!("Template with ID '{}' already exists", template.id));
        }
        Ok(None) => {
            // ID is available, proceed
        }
        Err(e) => {
            error_log!("[Templates] Database error checking template: {}", e);
            return ApiResult::error(format!("Database error: {}", e));
        }
    }

    // Save the new template
    let template_id = template.id.clone();
    match with_database(|db| db.save_template(&template)) {
        Ok(_) => {
            info_log!("[Templates] Created template: {}", template_id);
            ApiResult::success(template_id)
        }
        Err(e) => {
            error_log!("[Templates] Failed to save template: {}", e);
            ApiResult::error(format!("Failed to save template: {}", e))
        }
    }
}

/// Update an existing template
#[tauri::command(rename_all = "snake_case")]
pub async fn update_template(template_id: String, template: Template) -> ApiResult<()> {
    info_log!("[Templates] Updating template: {}", template_id);

    // Verify template exists
    match with_database(|db| db.load_template(&template_id)) {
        Ok(Some(_)) => {
            // Template exists, proceed with update
        }
        Ok(None) => {
            error_log!("[Templates] Template not found: {}", template_id);
            return ApiResult::error(format!("Template '{}' not found", template_id));
        }
        Err(e) => {
            error_log!("[Templates] Database error: {}", e);
            return ApiResult::error(format!("Database error: {}", e));
        }
    }

    // Save updated template
    match with_database(|db| db.save_template(&template)) {
        Ok(_) => {
            info_log!("[Templates] Updated template: {}", template_id);
            ApiResult::success(())
        }
        Err(e) => {
            error_log!("[Templates] Failed to update template: {}", e);
            ApiResult::error(format!("Failed to update template: {}", e))
        }
    }
}

/// Delete a template (blocked if jobs are using it)
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_template(template_id: String) -> ApiResult<()> {
    info_log!("[Templates] Deleting template: {}", template_id);

    // Check if any jobs are using this template
    let job_count = match with_database(|db| db.count_jobs_using_template(&template_id)) {
        Ok(count) => count,
        Err(e) => {
            error_log!("[Templates] Failed to count jobs using template: {}", e);
            return ApiResult::error(format!("Database error: {}", e));
        }
    };

    if job_count > 0 {
        error_log!("[Templates] Cannot delete template {} - {} jobs are using it", template_id, job_count);
        return ApiResult::error(format!("Cannot delete template: {} job(s) are using it", job_count));
    }

    match with_database(|db| db.delete_template(&template_id)) {
        Ok(true) => {
            info_log!("[Templates] Deleted template: {}", template_id);
            ApiResult::success(())
        }
        Ok(false) => {
            error_log!("[Templates] Template not found: {}", template_id);
            ApiResult::error(format!("Template '{}' not found", template_id))
        }
        Err(e) => {
            error_log!("[Templates] Database error: {}", e);
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
    let template = match with_database(|db| db.load_template(&template_id)) {
        Ok(Some(t)) => t,
        Ok(None) => {
            error_log!("[Templates] Template not found: {}", template_id);
            return ValidationResult {
                is_valid: false,
                issues: vec![format!("Template '{}' not found", template_id)],
                warnings: vec![],
                suggestions: vec![],
            };
        }
        Err(e) => {
            error_log!("[Templates] Database error: {}", e);
            return ValidationResult {
                is_valid: false,
                issues: vec![format!("Database error: {}", e)],
                warnings: vec![],
                suggestions: vec![],
            };
        }
    };

    // Use extracted validation logic (testable without database)
    let result = crate::templates::validate_values(&template, &values);

    if !result.is_valid {
        error_log!("[Templates] Validation failed with {} issues", result.issues.len());
    }

    result
}

/// Preview NAMD config with user values
#[tauri::command(rename_all = "snake_case")]
pub async fn preview_namd_config(
    template_id: String,
    values: HashMap<String, Value>,
) -> ApiResult<String> {
    info_log!("[Templates] Previewing NAMD config for template: {}", template_id);

    // Load template
    let template = match with_database(|db| db.load_template(&template_id)) {
        Ok(Some(t)) => t,
        Ok(None) => {
            return ApiResult::error(format!("Template '{}' not found", template_id));
        }
        Err(e) => {
            return ApiResult::error(format!("Database error: {}", e));
        }
    };

    // Render template
    match crate::templates::render_template(&template, &values) {
        Ok(rendered) => {
            info_log!("[Templates] Preview generated successfully");
            ApiResult::success(rendered)
        }
        Err(e) => {
            error_log!("[Templates] Preview render failed: {}", e);
            ApiResult::error(format!("Rendering error: {}", e))
        }
    }
}

/// Preview template with default/sample values (for template editor testing)
#[tauri::command(rename_all = "snake_case")]
pub async fn preview_template_with_defaults(template_id: String) -> ApiResult<String> {
    info_log!("[Templates] Previewing template with defaults: {}", template_id);

    // Load template
    let template = match with_database(|db| db.load_template(&template_id)) {
        Ok(Some(t)) => t,
        Ok(None) => {
            return ApiResult::error(format!("Template '{}' not found", template_id));
        }
        Err(e) => {
            return ApiResult::error(format!("Database error: {}", e));
        }
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
            info_log!("[Templates] Preview with defaults generated successfully");
            ApiResult::success(rendered)
        }
        Err(e) => {
            error_log!("[Templates] Preview with defaults failed: {}", e);
            ApiResult::error(format!("Rendering error: {}", e))
        }
    }
}
