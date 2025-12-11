pub mod types;
pub mod renderer;

pub use types::{Template, VariableDefinition, VariableType, TemplateSummary};
pub use renderer::render_template;
