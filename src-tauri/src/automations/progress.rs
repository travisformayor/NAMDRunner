/// Structured progress reporting for automation workflows
/// Provides detailed progress information with percentages and step tracking
use serde::{Deserialize, Serialize};

/// Structured progress information for automation workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    /// Current step being executed
    pub step: String,
    /// Current step number (1-based)
    pub current_step: u32,
    /// Total number of steps
    pub total_steps: u32,
    /// Progress percentage (0-100)
    pub percentage: f32,
    /// Optional detailed message
    pub message: Option<String>,
    /// Optional sub-progress for current step (0-100)
    pub sub_progress: Option<f32>,
}

impl ProgressInfo {
    /// Create a new progress info
    pub fn new(step: &str, current_step: u32, total_steps: u32) -> Self {
        let percentage = if total_steps > 0 {
            ((current_step - 1) as f32 / total_steps as f32) * 100.0
        } else {
            0.0
        };

        ProgressInfo {
            step: step.to_string(),
            current_step,
            total_steps,
            percentage: percentage.min(100.0),
            message: None,
            sub_progress: None,
        }
    }

    /// Create progress with a detailed message
    pub fn with_message(step: &str, current_step: u32, total_steps: u32, message: &str) -> Self {
        let mut progress = Self::new(step, current_step, total_steps);
        progress.message = Some(message.to_string());
        progress
    }

    /// Create progress with sub-step progress
    pub fn with_sub_progress(step: &str, current_step: u32, total_steps: u32, sub_progress: f32) -> Self {
        let mut progress = Self::new(step, current_step, total_steps);
        progress.sub_progress = Some(sub_progress.clamp(0.0, 100.0));
        progress
    }

    /// Update the sub-progress for the current step
    pub fn update_sub_progress(&mut self, sub_progress: f32) {
        self.sub_progress = Some(sub_progress.clamp(0.0, 100.0));
    }

    /// Update the message for the current step
    pub fn update_message(&mut self, message: &str) {
        self.message = Some(message.to_string());
    }

    /// Get a human-readable progress string
    pub fn to_display_string(&self) -> String {
        let base = format!("Step {}/{}: {} ({}%)",
            self.current_step,
            self.total_steps,
            self.step,
            self.percentage as u32
        );

        if let Some(sub_progress) = self.sub_progress {
            format!("{} - {}%", base, sub_progress as u32)
        } else if let Some(message) = &self.message {
            format!("{} - {}", base, message)
        } else {
            base
        }
    }

    /// Get a simple text-only version (for compatibility with existing callbacks)
    pub fn to_simple_string(&self) -> String {
        if let Some(message) = &self.message {
            format!("{}: {}", self.step, message)
        } else {
            self.step.clone()
        }
    }
}

/// Progress callback trait for structured progress reporting
pub trait ProgressCallback {
    /// Report structured progress
    fn report_progress(&self, progress: &ProgressInfo);

    /// Report simple text progress (for compatibility)
    fn report_text(&self, text: &str);
}

/// Implementation for simple function callbacks (existing pattern)
impl<F> ProgressCallback for F
where
    F: Fn(&str),
{
    fn report_progress(&self, progress: &ProgressInfo) {
        self(&progress.to_simple_string());
    }

    fn report_text(&self, text: &str) {
        self(text);
    }
}

/// Progress tracker for automation workflows
pub struct ProgressTracker {
    total_steps: u32,
    current_step: u32,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(total_steps: u32) -> Self {
        ProgressTracker {
            total_steps,
            current_step: 0,
        }
    }

    /// Start the next step and return progress info
    pub fn next_step(&mut self, step: &str) -> ProgressInfo {
        self.current_step += 1;
        ProgressInfo::new(step, self.current_step, self.total_steps)
    }

    /// Start the next step with a message
    pub fn next_step_with_message(&mut self, step: &str, message: &str) -> ProgressInfo {
        self.current_step += 1;
        ProgressInfo::with_message(step, self.current_step, self.total_steps, message)
    }

    /// Get current progress info without advancing
    pub fn current_progress(&self, step: &str) -> ProgressInfo {
        ProgressInfo::new(step, self.current_step, self.total_steps)
    }

    /// Update current step with sub-progress
    pub fn update_sub_progress(&self, step: &str, sub_progress: f32) -> ProgressInfo {
        ProgressInfo::with_sub_progress(step, self.current_step, self.total_steps, sub_progress)
    }

    /// Check if all steps are complete
    pub fn is_complete(&self) -> bool {
        self.current_step >= self.total_steps
    }

    /// Get completion progress (100% when done)
    pub fn completion_progress(&self) -> ProgressInfo {
        ProgressInfo::new("Complete", self.total_steps, self.total_steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage_calculation() {
        // Test business logic: percentage calculation formula
        let progress = ProgressInfo::new("Testing", 2, 5);
        assert_eq!(progress.percentage, 20.0); // (2-1)/5 * 100

        let progress_first = ProgressInfo::new("Start", 1, 4);
        assert_eq!(progress_first.percentage, 0.0); // (1-1)/4 * 100

        let progress_last = ProgressInfo::new("End", 4, 4);
        assert_eq!(progress_last.percentage, 75.0); // (4-1)/4 * 100
    }

    #[test]
    fn test_display_string_formatting() {
        // Test business logic: string formatting with message
        let progress = ProgressInfo::with_message("Uploading", 1, 3, "file.pdb");
        assert_eq!(progress.to_display_string(), "Step 1/3: Uploading (0%) - file.pdb");

        // Test business logic: string formatting with sub-progress
        let progress_sub = ProgressInfo::with_sub_progress("Copying", 2, 4, 75.0);
        assert_eq!(progress_sub.to_display_string(), "Step 2/4: Copying (25%) - 75%");
    }

    #[test]
    fn test_progress_tracker_percentage_progression() {
        // Test business logic: percentage increases correctly as steps progress
        let mut tracker = ProgressTracker::new(3);

        let step1 = tracker.next_step("Initialize");
        assert_eq!(step1.percentage, 0.0);

        let step2 = tracker.next_step("Process");
        assert!((step2.percentage - 33.333333).abs() < 0.001, "Step 2 percentage should be ~33.33%, got {}", step2.percentage);

        let step3 = tracker.next_step("Complete");
        assert!((step3.percentage - 66.666667).abs() < 0.001, "Step 3 percentage should be ~66.67%, got {}", step3.percentage);

        // Test business logic: tracker knows when all steps are complete
        assert!(tracker.is_complete());
    }

    #[test]
    fn test_simple_string_conversion() {
        let progress = ProgressInfo::with_message("Testing", 1, 1, "details");
        assert_eq!(progress.to_simple_string(), "Testing: details");

        let progress_no_msg = ProgressInfo::new("Testing", 1, 1);
        assert_eq!(progress_no_msg.to_simple_string(), "Testing");
    }
}