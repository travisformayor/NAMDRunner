pub mod mode;
pub mod state;

#[allow(unused_imports)]
pub use mode::{is_demo_mode, set_demo_mode, execute_with_mode};
pub use state::{with_demo_state, get_demo_state, advance_demo_progression};
