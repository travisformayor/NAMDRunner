use crate::types::*;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::Utc;

lazy_static::lazy_static! {
    static ref DEMO_STATE: Mutex<DemoStateManager> = Mutex::new(DemoStateManager::new());
}

#[derive(Debug, Clone)]
pub struct DemoBehavior {
    pub connection_latency: u64,
    pub file_operation_latency: u64,
    pub slurm_latency: u64,
}

impl Default for DemoBehavior {
    fn default() -> Self {
        Self {
            connection_latency: 300,
            file_operation_latency: 200,
            slurm_latency: 150,
        }
    }
}

#[derive(Debug)]
pub struct DemoStateManager {
    pub connection_state: ConnectionState,
    pub session_info: Option<SessionInfo>,
    pub jobs: HashMap<String, JobInfo>,
    pub job_counter: u32,
    pub demo_behavior: DemoBehavior,
}

impl DemoStateManager {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            session_info: None,
            jobs: HashMap::new(),
            job_counter: 0,
            demo_behavior: DemoBehavior::default(),
        }
    }

    #[allow(dead_code)]
    fn create_demo_project_path(job_id: &str) -> String {
        format!("/demo/projects/testuser/namdrunner_jobs/{}", job_id)
    }

    #[allow(dead_code)]
    fn create_demo_scratch_path(job_id: &str) -> String {
        format!("/demo/scratch/testuser/namdrunner_jobs/{}", job_id)
    }

    #[allow(dead_code)]
    fn create_demo_job(&mut self, name: &str, status: JobStatus, slurm_job_id: Option<&str>) -> JobInfo {
        self.job_counter += 1;
        let job_id = format!("job_{:03}", self.job_counter);
        let now = Utc::now().to_rfc3339();

        let mut job_info = JobInfo::new(
            job_id.clone(),
            name.to_string(),
            NAMDConfig {
                outputname: "output".to_string(),
                temperature: 300.0,
                timestep: 2.0,
                execution_mode: ExecutionMode::Run,
                steps: 1000,
                cell_basis_vector1: None,
                cell_basis_vector2: None,
                cell_basis_vector3: None,
                pme_enabled: false,
                npt_enabled: false,
                langevin_damping: 5.0,
                xst_freq: 100,
                output_energies_freq: 100,
                dcd_freq: 100,
                restart_freq: 500,
                output_pressure_freq: 100,
            },
            SlurmConfig {
                cores: 4,
                memory: "4GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: Some("compute".to_string()),
                qos: None,
            },
            Vec::new(),
            Self::create_demo_project_path(&job_id),
        );

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
        job_info.project_dir = Some(Self::create_demo_project_path(&job_id));
        job_info.scratch_dir = Some(Self::create_demo_scratch_path(&job_id));

        job_info
    }

    pub fn advance_job_states(&mut self) {
        let now = Utc::now().to_rfc3339();

        for job in self.jobs.values_mut() {
            match job.status {
                JobStatus::Pending => {
                    job.status = JobStatus::Running;
                    job.updated_at = Some(now.clone());
                },
                JobStatus::Running => {
                    job.status = JobStatus::Completed;
                    job.updated_at = Some(now.clone());
                    job.completed_at = Some(now.clone());
                },
                _ => {}
            }
        }
    }

    pub fn should_simulate_error(&self) -> bool {
        false
    }

    pub fn get_delay(&self, operation: &str) -> u64 {
        match operation {
            "connection" => self.demo_behavior.connection_latency,
            "file" => self.demo_behavior.file_operation_latency,
            "slurm" => self.demo_behavior.slurm_latency,
            _ => 300,
        }
    }

    pub fn generate_slurm_job_id(&self) -> String {
        let base = 12345000;
        (base + self.job_counter).to_string()
    }
}

pub fn with_demo_state<T, F>(f: F) -> Option<T>
where
    F: FnOnce(&mut DemoStateManager) -> T,
{
    DEMO_STATE.lock().ok().map(|mut state| f(&mut state))
}

pub fn get_demo_state<T, F>(f: F) -> Option<T>
where
    F: FnOnce(&DemoStateManager) -> T,
{
    DEMO_STATE.lock().ok().map(|state| f(&state))
}

pub fn advance_demo_progression() {
    if let Ok(mut state) = DEMO_STATE.lock() {
        state.advance_job_states();
    }
}
