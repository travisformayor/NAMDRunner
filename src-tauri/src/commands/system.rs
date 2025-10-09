use crate::types::*;
use crate::demo::mode::set_demo_mode;

/// Set application mode (demo/real)
#[tauri::command(rename_all = "snake_case")]
pub async fn set_app_mode(is_demo: bool) -> ApiResult<String> {
    set_demo_mode(is_demo);
    let mode_str = if is_demo { "demo" } else { "real" };
    println!("[MODE] App mode set to: {}", mode_str);
    ApiResult::success(format!("Mode set to {}", mode_str))
}
