use crate::types::*;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::Utc;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref MOCK_STATE: Mutex<MockStateManager> = Mutex::new(MockStateManager::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestScenario {
    CleanSlate,
    WithSampleJobs,
    WithActiveJobs,
}

#[derive(Debug, Clone)]
pub struct MockBehavior {
    pub connection_latency: u64,
    pub file_operation_latency: u64,
    pub slurm_latency: u64,
}

impl Default for MockBehavior {
    fn default() -> Self {
        Self {
            connection_latency: 300,
            file_operation_latency: 200,
            slurm_latency: 150,
        }
    }
}

#[derive(Debug)]
pub struct MockStateManager {
    // Connection state
    pub connection_state: ConnectionState,
    pub session_info: Option<SessionInfo>,
    
    // Job state
    pub jobs: HashMap<String, JobInfo>,
    pub job_counter: u32,
    
    // Simple mock configuration
    pub current_scenario: TestScenario,
    pub mock_behavior: MockBehavior,
}

impl MockStateManager {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            session_info: None,
            jobs: HashMap::new(),
            job_counter: 0,
            current_scenario: TestScenario::CleanSlate,
            mock_behavior: MockBehavior::default(),
        }
    }

    pub fn reset_to_clean_state(&mut self) {
        *self = Self::new();
    }

    pub fn load_test_scenario(&mut self, scenario: TestScenario) {
        self.current_scenario = scenario.clone();
        self.reset_to_clean_state();

        match scenario {
            TestScenario::CleanSlate => {
                // Empty state - no additional setup needed
            },

            TestScenario::WithSampleJobs => {
                self.add_sample_jobs();
            },

            TestScenario::WithActiveJobs => {
                self.connection_state = ConnectionState::Connected;
                self.session_info = Some(SessionInfo {
                    host: "login.rc.colorado.edu".to_string(),
                    username: "testuser".to_string(),
                    connected_at: Utc::now().to_rfc3339(),
                });
                self.add_active_jobs();
            },
        }
    }

    // Simple job generation helpers
    fn add_sample_jobs(&mut self) {
        let jobs = vec![
            self.create_job("sample_simulation", JobStatus::Completed, Some("12345680")),
            self.create_job("another_simulation", JobStatus::Failed, Some("12345681")),
            self.create_job("test_simulation", JobStatus::Created, None),
        ];

        for job in jobs {
            self.jobs.insert(job.job_id.clone(), job);
        }
    }

    fn add_active_jobs(&mut self) {
        let jobs = vec![
            self.create_job("running_simulation", JobStatus::Running, Some("12345679")),
            self.create_job("pending_simulation", JobStatus::Pending, Some("12345678")),
        ];

        for job in jobs {
            self.jobs.insert(job.job_id.clone(), job);
        }
    }

    fn create_job(&mut self, name: &str, status: JobStatus, slurm_job_id: Option<&str>) -> JobInfo {
        self.job_counter += 1;
        let job_id = format!("job_{:03}", self.job_counter);
        let now = Utc::now().to_rfc3339();
        
        let mut job_info = JobInfo::new(
            job_id.clone(),
            name.to_string(),
            NAMDConfig {
                steps: 1000,
                temperature: 300.0,
                timestep: 2.0,
                outputname: "output".to_string(),
                dcd_freq: Some(100),
                restart_freq: Some(500),
            },
            SlurmConfig {
                cores: 4,
                memory: "4GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: Some("compute".to_string()),
                qos: None,
            },
            Vec::new(),
            format!("/projects/testuser/namdrunner_jobs/{}", job_id),
        );

        // Override with specific mock values
        job_info.status = status.clone();
        job_info.slurm_job_id = slurm_job_id.map(|s| s.to_string());
        job_info.created_at = now.clone();
        job_info.updated_at = Some(now.clone());
        job_info.submitted_at = if slurm_job_id.is_some() { Some(now.clone()) } else { None };
        job_info.completed_at = if matches!(status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
            Some(now)
        } else {
            None
        };
        job_info.project_dir = Some(format!("/projects/testuser/namdrunner_jobs/{}", job_id));
        job_info.scratch_dir = Some(format!("/scratch/alpine/testuser/namdrunner_jobs/{}", job_id));

        job_info
    }

    // Simple state progression - just cycle through predictable states
    pub fn advance_job_states(&mut self) {
        let now = Utc::now().to_rfc3339();

        for job in self.jobs.values_mut() {
            match job.status {
                JobStatus::Pending => {
                    job.status = JobStatus::Running;
                    job.updated_at = Some(now.clone());
                },
                JobStatus::Running => {
                    // Deterministic completion for testing
                    job.status = JobStatus::Completed;
                    job.updated_at = Some(now.clone());
                    job.completed_at = Some(now.clone());
                },
                _ => {} // No progression for other states
            }
        }
    }

    // Simple error simulation - always returns false for predictable testing
    pub fn should_simulate_error(&self) -> bool {
        false
    }

    pub fn get_delay(&self, operation: &str) -> u64 {
        match operation {
            "connection" => self.mock_behavior.connection_latency,
            "file" => self.mock_behavior.file_operation_latency,
            "slurm" => self.mock_behavior.slurm_latency,
            _ => 300,
        }
    }

    // Generate predictable SLURM job ID for testing
    pub fn generate_slurm_job_id(&self) -> String {
        let base = 12345000;
        (base + self.job_counter).to_string()
    }
}

// Global state access functions
pub fn with_mock_state<T, F>(f: F) -> Option<T>
where
    F: FnOnce(&mut MockStateManager) -> T,
{
    MOCK_STATE.lock().ok().map(|mut state| f(&mut state))
}

pub fn get_mock_state<T, F>(f: F) -> Option<T>
where
    F: FnOnce(&MockStateManager) -> T,
{
    MOCK_STATE.lock().ok().map(|state| f(&state))
}

// Simple test scenario management
pub fn set_test_scenario(scenario: TestScenario) {
    if let Ok(mut state) = MOCK_STATE.lock() {
        state.load_test_scenario(scenario);
    }
}

pub fn reset_mock_state() {
    if let Ok(mut state) = MOCK_STATE.lock() {
        state.reset_to_clean_state();
    }
}

pub fn advance_job_progression() {
    if let Ok(mut state) = MOCK_STATE.lock() {
        state.advance_job_states();
    }
}