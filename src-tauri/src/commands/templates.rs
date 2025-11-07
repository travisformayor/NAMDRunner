use crate::types::*;
use crate::database::with_database;
use crate::templates::Template;
use crate::{info_log, error_log};
use std::collections::HashMap;
use serde_json::Value;

/// List all available templates (returns summary info)
#[tauri::command(rename_all = "snake_case")]
pub async fn list_templates() -> ListTemplatesResult {
    info_log!("[Templates] Listing all templates from database");

    // Load default templates if this is the first call and DB is empty
    // This ensures defaults are available and logging works (frontend is connected)
    if let Err(e) = crate::database::ensure_default_templates_loaded() {
        error_log!("[Templates] Failed to ensure default templates: {}", e);
    }

    match with_database(|db| db.list_templates()) {
        Ok(templates) => {
            info_log!("[Templates] Found {} templates in DB", templates.len());
            ListTemplatesResult {
                success: true,
                templates: Some(templates),
                error: None,
            }
        }
        Err(e) => {
            error_log!("[Templates] Failed to list templates: {}", e);
            ListTemplatesResult {
                success: false,
                templates: None,
                error: Some(format!("Database error: {}", e)),
            }
        }
    }
}

/// Get full template definition by ID
#[tauri::command(rename_all = "snake_case")]
pub async fn get_template(template_id: String) -> GetTemplateResult {
    info_log!("[Templates] Loading template: {}", template_id);

    match with_database(|db| db.load_template(&template_id)) {
        Ok(Some(template)) => {
            info_log!("[Templates] Loaded template: {}", template.name);
            GetTemplateResult {
                success: true,
                template: Some(template),
                error: None,
            }
        }
        Ok(None) => {
            error_log!("[Templates] Template not found: {}", template_id);
            GetTemplateResult {
                success: false,
                template: None,
                error: Some(format!("Template '{}' not found", template_id)),
            }
        }
        Err(e) => {
            error_log!("[Templates] Database error: {}", e);
            GetTemplateResult {
                success: false,
                template: None,
                error: Some(format!("Database error: {}", e)),
            }
        }
    }
}

/// Create a new template
#[tauri::command(rename_all = "snake_case")]
pub async fn create_template(template: Template) -> CreateTemplateResult {
    info_log!("[Templates] Creating template: {}", template.id);

    // Check if template ID already exists
    match with_database(|db| db.load_template(&template.id)) {
        Ok(Some(_)) => {
            error_log!("[Templates] Template ID already exists: {}", template.id);
            return CreateTemplateResult {
                success: false,
                template_id: None,
                error: Some(format!("Template with ID '{}' already exists", template.id)),
            };
        }
        Ok(None) => {
            // ID is available, proceed
        }
        Err(e) => {
            error_log!("[Templates] Database error checking template: {}", e);
            return CreateTemplateResult {
                success: false,
                template_id: None,
                error: Some(format!("Database error: {}", e)),
            };
        }
    }

    // Save the new template
    match with_database(|db| db.save_template(&template)) {
        Ok(_) => {
            info_log!("[Templates] Created template: {}", template.id);
            CreateTemplateResult {
                success: true,
                template_id: Some(template.id),
                error: None,
            }
        }
        Err(e) => {
            error_log!("[Templates] Failed to save template: {}", e);
            CreateTemplateResult {
                success: false,
                template_id: None,
                error: Some(format!("Failed to save template: {}", e)),
            }
        }
    }
}

/// Update an existing template
#[tauri::command(rename_all = "snake_case")]
pub async fn update_template(template_id: String, template: Template) -> UpdateTemplateResult {
    info_log!("[Templates] Updating template: {}", template_id);

    // Verify template exists
    match with_database(|db| db.load_template(&template_id)) {
        Ok(Some(_)) => {
            // Template exists, proceed with update
        }
        Ok(None) => {
            error_log!("[Templates] Template not found: {}", template_id);
            return UpdateTemplateResult {
                success: false,
                error: Some(format!("Template '{}' not found", template_id)),
            };
        }
        Err(e) => {
            error_log!("[Templates] Database error: {}", e);
            return UpdateTemplateResult {
                success: false,
                error: Some(format!("Database error: {}", e)),
            };
        }
    }

    // Save updated template
    match with_database(|db| db.save_template(&template)) {
        Ok(_) => {
            info_log!("[Templates] Updated template: {}", template_id);
            UpdateTemplateResult {
                success: true,
                error: None,
            }
        }
        Err(e) => {
            error_log!("[Templates] Failed to update template: {}", e);
            UpdateTemplateResult {
                success: false,
                error: Some(format!("Failed to update template: {}", e)),
            }
        }
    }
}

/// Delete a template (blocked if jobs are using it)
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_template(template_id: String) -> DeleteTemplateResult {
    info_log!("[Templates] Deleting template: {}", template_id);

    // Check if any jobs are using this template
    let job_count = match with_database(|db| db.count_jobs_using_template(&template_id)) {
        Ok(count) => count,
        Err(e) => {
            error_log!("[Templates] Failed to count jobs using template: {}", e);
            return DeleteTemplateResult {
                success: false,
                error: Some(format!("Database error: {}", e)),
            };
        }
    };

    if job_count > 0 {
        error_log!("[Templates] Cannot delete template {} - {} jobs are using it", template_id, job_count);
        return DeleteTemplateResult {
            success: false,
            error: Some(format!("Cannot delete template: {} job(s) are using it", job_count)),
        };
    }

    match with_database(|db| db.delete_template(&template_id)) {
        Ok(true) => {
            info_log!("[Templates] Deleted template: {}", template_id);
            DeleteTemplateResult {
                success: true,
                error: None,
            }
        }
        Ok(false) => {
            error_log!("[Templates] Template not found: {}", template_id);
            DeleteTemplateResult {
                success: false,
                error: Some(format!("Template '{}' not found", template_id)),
            }
        }
        Err(e) => {
            error_log!("[Templates] Database error: {}", e);
            DeleteTemplateResult {
                success: false,
                error: Some(format!("Database error: {}", e)),
            }
        }
    }
}

/// Validate template values against template definition
#[tauri::command(rename_all = "snake_case")]
pub async fn validate_template_values(
    template_id: String,
    values: HashMap<String, Value>,
) -> ValidateTemplateValuesResult {
    info_log!("[Templates] Validating values for template: {}", template_id);

    // Load template
    let template = match with_database(|db| db.load_template(&template_id)) {
        Ok(Some(t)) => t,
        Ok(None) => {
            error_log!("[Templates] Template not found: {}", template_id);
            return ValidateTemplateValuesResult {
                valid: false,
                errors: vec![format!("Template '{}' not found", template_id)],
            };
        }
        Err(e) => {
            error_log!("[Templates] Database error: {}", e);
            return ValidateTemplateValuesResult {
                valid: false,
                errors: vec![format!("Database error: {}", e)],
            };
        }
    };

    // Use extracted validation logic (testable without database)
    let errors = crate::templates::validate_values(&template, &values);
    let valid = errors.is_empty();
    if valid {
        info_log!("[Templates] Validation passed for template: {}", template_id);
    } else {
        error_log!("[Templates] Validation failed with {} errors", errors.len());
    }

    ValidateTemplateValuesResult {
        valid,
        errors,
    }
}

/// Preview NAMD config with user values
#[tauri::command(rename_all = "snake_case")]
pub async fn preview_namd_config(
    template_id: String,
    values: HashMap<String, Value>,
) -> PreviewResult {
    info_log!("[Templates] Previewing NAMD config for template: {}", template_id);

    // Load template
    let template = match with_database(|db| db.load_template(&template_id)) {
        Ok(Some(t)) => t,
        Ok(None) => {
            return PreviewResult {
                success: false,
                content: None,
                error: Some(format!("Template '{}' not found", template_id)),
            };
        }
        Err(e) => {
            return PreviewResult {
                success: false,
                content: None,
                error: Some(format!("Database error: {}", e)),
            };
        }
    };

    // Render template
    match crate::templates::render_template(&template, &values) {
        Ok(rendered) => {
            info_log!("[Templates] Preview generated successfully");
            PreviewResult {
                success: true,
                content: Some(rendered),
                error: None,
            }
        }
        Err(e) => {
            error_log!("[Templates] Preview render failed: {}", e);
            PreviewResult {
                success: false,
                content: None,
                error: Some(format!("Rendering error: {}", e)),
            }
        }
    }
}
