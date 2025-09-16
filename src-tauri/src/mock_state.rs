use crate::types::*;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref MOCK_STATE: Mutex<MockStateManager> = Mutex::new(MockStateManager::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestScenario {
    FreshInstall,
    ReturningUser,
    ActiveSession,
    NetworkIssues,
    SystemErrors,
    HeavyLoad,
    EdgeCases,
    JobLifecycle,
    CleanSlate,
}

#[derive(Debug, Clone)]
pub struct MockBehavior {
    pub connection_latency: u64,
    pub file_operation_latency: u64,
    pub slurm_latency: u64,
    pub error_rate: f64, // 0.0 to 1.0
}

impl Default for MockBehavior {
    fn default() -> Self {
        Self {
            connection_latency: 500,
            file_operation_latency: 300,
            slurm_latency: 200,
            error_rate: 0.0,
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
    
    // Test scenario configuration
    #[allow(dead_code)]
    pub current_scenario: TestScenario,
    pub mock_behavior: MockBehavior,
    
    // State progression tracking
    #[allow(dead_code)]
    pub job_progression: HashMap<String, usize>, // job_id -> progression_index
    pub last_sync: Option<DateTime<Utc>>,
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
            job_progression: HashMap::new(),
            last_sync: None,
        }
    }

    #[allow(dead_code)]
    pub fn reset_to_clean_state(&mut self) {
        *self = Self::new();
    }

    #[allow(dead_code)]
    pub fn load_test_scenario(&mut self, scenario: TestScenario) {
        self.current_scenario = scenario.clone();
        self.reset_to_clean_state();
        
        match scenario {
            TestScenario::FreshInstall => {
                // No additional setup needed
            },
            
            TestScenario::ReturningUser => {
                self.add_sample_jobs();
                self.mock_behavior.error_rate = 0.05;
            },
            
            TestScenario::ActiveSession => {
                self.connection_state = ConnectionState::Connected;
                self.session_info = Some(SessionInfo {
                    host: "login.rc.colorado.edu".to_string(),
                    username: "testuser".to_string(),
                    connected_at: Utc::now().to_rfc3339(),
                });
                self.add_active_jobs();
                self.mock_behavior.error_rate = 0.02;
            },
            
            TestScenario::NetworkIssues => {
                self.connection_state = ConnectionState::Expired;
                self.mock_behavior.connection_latency = 5000;
                self.mock_behavior.error_rate = 0.3;
            },
            
            TestScenario::SystemErrors => {
                self.connection_state = ConnectionState::Connected;
                self.session_info = Some(SessionInfo {
                    host: "maintenance.cluster.edu".to_string(),
                    username: "testuser".to_string(),
                    connected_at: (Utc::now() - chrono::Duration::minutes(10)).to_rfc3339(),
                });
                self.add_failed_jobs();
                self.mock_behavior.error_rate = 0.4;
            },
            
            TestScenario::HeavyLoad => {
                self.connection_state = ConnectionState::Connected;
                self.session_info = Some(SessionInfo {
                    host: "login.rc.colorado.edu".to_string(),
                    username: "power_user".to_string(),
                    connected_at: (Utc::now() - chrono::Duration::hours(2)).to_rfc3339(),
                });
                self.add_many_jobs();
                self.mock_behavior.file_operation_latency = 800;
                self.mock_behavior.error_rate = 0.1;
            },
            
            TestScenario::EdgeCases => {
                self.connection_state = ConnectionState::Connected;
                self.session_info = Some(SessionInfo {
                    host: "login.rc.colorado.edu".to_string(),
                    username: "tëst_üser".to_string(), // Unicode username
                    connected_at: Utc::now().to_rfc3339(),
                });
                self.add_edge_case_jobs();
                self.mock_behavior.error_rate = 0.15;
            },
            
            TestScenario::JobLifecycle => {
                self.connection_state = ConnectionState::Connected;
                self.session_info = Some(SessionInfo {
                    host: "login.rc.colorado.edu".to_string(),
                    username: "testuser".to_string(),
                    connected_at: Utc::now().to_rfc3339(),
                });
                self.add_job_for_lifecycle_testing();
            },
            
            TestScenario::CleanSlate => {
                // Already reset above
            },
        }
    }

    // Job generation helpers
    #[allow(dead_code)]
    fn add_sample_jobs(&mut self) {
        let jobs = vec![
            self.create_job("completed_simulation", JobStatus::Completed, Some("12345680")),
            self.create_job("failed_simulation", JobStatus::Failed, Some("12345681")),
            self.create_job("cancelled_simulation", JobStatus::Cancelled, Some("12345682")),
            self.create_job("fresh_simulation", JobStatus::Created, None),
        ];
        
        for job in jobs {
            self.jobs.insert(job.job_id.clone(), job);
        }
    }

    #[allow(dead_code)]
    fn add_active_jobs(&mut self) {
        let jobs = vec![
            self.create_job("running_simulation", JobStatus::Running, Some("12345679")),
            self.create_job("pending_simulation", JobStatus::Pending, Some("12345678")),
            self.create_job("completed_simulation", JobStatus::Completed, Some("12345680")),
        ];
        
        for job in jobs {
            self.jobs.insert(job.job_id.clone(), job);
        }
    }

    #[allow(dead_code)]
    fn add_failed_jobs(&mut self) {
        let jobs = vec![
            self.create_job("failed_simulation", JobStatus::Failed, Some("12345681")),
            self.create_job("cancelled_simulation", JobStatus::Cancelled, Some("12345682")),
        ];
        
        for job in jobs {
            self.jobs.insert(job.job_id.clone(), job);
        }
    }

    #[allow(dead_code)]
    fn add_many_jobs(&mut self) {
        for i in 1..=25 {
            let status = match i % 5 {
                0 => JobStatus::Completed,
                1 => JobStatus::Running,
                2 => JobStatus::Pending,
                3 => JobStatus::Failed,
                _ => JobStatus::Created,
            };
            
            let slurm_id = if matches!(status, JobStatus::Created) {
                None
            } else {
                Some(format!("{}", 12345000 + i))
            };
            
            let job = self.create_job(
                &format!("batch_test_{}", i),
                status,
                slurm_id.as_deref(),
            );
            
            self.jobs.insert(job.job_id.clone(), job);
        }
    }

    #[allow(dead_code)]
    fn add_edge_case_jobs(&mut self) {
        let jobs = vec![
            self.create_job("test-job_v2.1_α-helix", JobStatus::Created, None),
            self.create_job("large_membrane_system", JobStatus::Running, Some("12345683")),
        ];
        
        for job in jobs {
            self.jobs.insert(job.job_id.clone(), job);
        }
    }

    #[allow(dead_code)]
    fn add_job_for_lifecycle_testing(&mut self) {
        let job = self.create_job("lifecycle_test", JobStatus::Created, None);
        self.jobs.insert(job.job_id.clone(), job);
    }

    #[allow(dead_code)]
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

    // State progression methods
    pub fn advance_job_states(&mut self) {
        let now = Utc::now().to_rfc3339();
        
        // Pre-calculate random values to avoid borrow checker issues
        let mut job_updates = Vec::new();
        
        for (job_id, job) in &self.jobs {
            match job.status {
                JobStatus::Pending => {
                    if rand::random::<f64>() < 0.3 { // 30% chance
                        job_updates.push((job_id.clone(), JobStatus::Running, None));
                    }
                },
                JobStatus::Running => {
                    if rand::random::<f64>() < 0.2 { // 20% chance
                        let failed = rand::random::<f64>() < 0.1; // 10% failure rate
                        let new_status = if failed { JobStatus::Failed } else { JobStatus::Completed };
                        job_updates.push((job_id.clone(), new_status, Some(now.clone())));
                    }
                },
                _ => {} // No progression for other states
            }
        }
        
        // Apply updates
        for (job_id, new_status, completed_at) in job_updates {
            if let Some(job) = self.jobs.get_mut(&job_id) {
                job.status = new_status;
                job.updated_at = Some(now.clone());
                if let Some(completed) = completed_at {
                    job.completed_at = Some(completed);
                }
            }
        }
        
        self.last_sync = Some(Utc::now());
    }


    pub fn should_simulate_error(&self) -> bool {
        rand::random::<f64>() < self.mock_behavior.error_rate
    }

    pub fn get_delay(&self, operation: &str) -> u64 {
        let base_delay = match operation {
            "connection" => self.mock_behavior.connection_latency,
            "file" => self.mock_behavior.file_operation_latency,
            "slurm" => self.mock_behavior.slurm_latency,
            _ => 500,
        };
        
        // Add some random variation (±20%)
        let variation = (base_delay as f64) * 0.2 * (rand::random::<f64>() - 0.5);
        std::cmp::max(100, (base_delay as f64 + variation) as u64)
    }

    // Generate realistic SLURM job ID
    pub fn generate_slurm_job_id(&self) -> String {
        let base = 10000000;
        let random = rand::random::<u32>() % 90000000;
        (base + random).to_string()
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

// Test scenario management
#[allow(dead_code)]
pub fn set_test_scenario(scenario: TestScenario) {
    if let Ok(mut state) = MOCK_STATE.lock() {
        state.load_test_scenario(scenario);
    }
}

#[allow(dead_code)]
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