/// NAMD configuration constants extracted from DNA origami tutorial
/// Source of truth: examples/origamiTutorial/step3/equil_min.namd

/// Force field parameters
pub const EXCLUDE: &str = "scaled1-4";
pub const SCALING_1_4: f64 = 1.0;
pub const SWITCH_DIST: f64 = 8.0;
pub const CUTOFF: f64 = 10.0;
pub const PAIRLIST_DIST: f64 = 12.0;

/// Integrator parameters
pub const NONBONDED_FREQ: u32 = 1;
pub const FULL_ELECT_FREQUENCY: u32 = 2;
pub const STEPS_PER_CYCLE: u32 = 12;

/// PME parameters
pub const PME_GRID_SPACING: f64 = 1.5;

/// Langevin piston (NPT) parameters
pub const LANGEVIN_PISTON_TARGET: f64 = 1.01325; // 1 atm in bar
pub const LANGEVIN_PISTON_PERIOD: f64 = 1000.0;
pub const LANGEVIN_PISTON_DECAY: f64 = 500.0;

/// Output wrap settings
pub const WRAP_ALL: bool = false;
pub const WRAP_WATER: bool = false;

/// Rigid bonds setting for 2fs timestep
pub const RIGID_BONDS: &str = "all";

/// Langevin hydrogen coupling (off for performance)
pub const LANGEVIN_HYDROGEN: bool = false;
