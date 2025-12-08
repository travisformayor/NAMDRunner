use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template defines a NAMD configuration template with variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub namd_config_template: String,  // NAMD config with {{variables}}
    pub variables: HashMap<String, VariableDefinition>,
    pub created_at: String,
    pub updated_at: String,
}

impl Template {
    /// Get list of FileUpload variable keys from template
    pub fn get_file_upload_keys(&self) -> Vec<String> {
        self.variables
            .iter()
            .filter_map(|(key, var_def)| {
                match var_def.var_type {
                    VariableType::FileUpload { .. } => Some(key.clone()),
                    _ => None,
                }
            })
            .collect()
    }
}

/// Variable definition describes a template variable's type and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDefinition {
    pub key: String,
    pub label: String,
    pub var_type: VariableType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_text: Option<String>,
}

/// Variable type enum with type-specific options
/// Uses externally-tagged format to match JSON files: {"FileUpload": {"extensions": [...]}}
/// All constraints and defaults are required (no silent fallbacks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    Number {
        min: f64,      // Required: minimum value constraint
        max: f64,      // Required: maximum value constraint
        default: f64,  // Required: default value
    },
    Text {
        default: String,  // Required: default text value
    },
    Boolean {
        default: bool,  // Required: default true/false
    },
    FileUpload {
        extensions: Vec<String>,  // e.g., [".psf", ".pdb"]
    },
}

/// Summary view of template for list display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub description: String,
}
