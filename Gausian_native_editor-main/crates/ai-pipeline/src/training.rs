/// Training orchestration and job management
/// Phase 4: Automatic LORA Creator
///
/// Manages training jobs, monitors progress, and coordinates with backends

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Training job identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct JobId(pub String);

impl JobId {
    /// Generate new random job ID
    pub fn new() -> Self {
        use sha2::{Sha256, Digest};
        use hex;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let input = format!("{}-{}", now, rand::random::<u64>());
        let hash = Sha256::digest(input.as_bytes());
        Self(hex::encode(&hash[..8]))
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

/// Training job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job queued, waiting to start
    Queued,
    /// Job is running
    Running,
    /// Job paused
    Paused,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job cancelled
    Cancelled,
}

/// Training job progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingProgress {
    /// Job ID
    pub job_id: JobId,

    /// Current status
    pub status: JobStatus,

    /// Current step (0-based)
    pub current_step: u64,

    /// Total steps
    pub total_steps: u64,

    /// Current loss value
    pub current_loss: Option<f32>,

    /// Average loss over recent steps
    pub avg_loss: Option<f32>,

    /// ETA in seconds
    pub eta_seconds: Option<u64>,

    /// Timestamp of last update
    pub updated_at: i64,

    /// Progress percentage (0-100)
    pub progress: f32,
}

impl TrainingProgress {
    /// Create new progress tracker
    pub fn new(job_id: JobId, total_steps: u64) -> Self {
        Self {
            job_id,
            status: JobStatus::Queued,
            current_step: 0,
            total_steps,
            current_loss: None,
            avg_loss: None,
            eta_seconds: None,
            updated_at: current_timestamp(),
            progress: 0.0,
        }
    }

    /// Update progress
    pub fn update(&mut self, current_step: u64, current_loss: Option<f32>) {
        self.current_step = current_step.min(self.total_steps);
        self.current_loss = current_loss;
        self.progress = (self.current_step as f32 / self.total_steps as f32) * 100.0;
        self.updated_at = current_timestamp();

        // Estimate ETA
        if current_step > 0 {
            let elapsed_secs = self.updated_at - SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            if elapsed_secs > 0 {
                let avg_step_time = elapsed_secs as f64 / current_step as f64;
                let remaining_steps = (self.total_steps - current_step) as f64;
                self.eta_seconds = Some((avg_step_time * remaining_steps) as u64);
            }
        }
    }

    /// Mark as started
    pub fn started(&mut self) {
        self.status = JobStatus::Running;
        self.updated_at = current_timestamp();
    }

    /// Mark as completed
    pub fn completed(&mut self) {
        self.status = JobStatus::Completed;
        self.current_step = self.total_steps;
        self.progress = 100.0;
        self.updated_at = current_timestamp();
    }

    /// Mark as failed
    pub fn failed(&mut self) {
        self.status = JobStatus::Failed;
        self.updated_at = current_timestamp();
    }

    /// Check if job is finished
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }
}

/// Training job metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJob {
    /// Job ID
    pub id: JobId,

    /// Job name
    pub name: String,

    /// Base model
    pub base_model: String,

    /// Number of training images
    pub num_images: usize,

    /// Number of training steps
    pub total_steps: u64,

    /// Output directory for weights
    pub output_dir: PathBuf,

    /// Backend used for training
    pub backend: String,

    /// Creation timestamp
    pub created_at: i64,

    /// Completion timestamp
    pub completed_at: Option<i64>,

    /// Error message if failed
    pub error: Option<String>,
}

impl TrainingJob {
    /// Create new training job
    pub fn new(
        name: String,
        base_model: String,
        num_images: usize,
        total_steps: u64,
        output_dir: PathBuf,
        backend: String,
    ) -> Self {
        Self {
            id: JobId::new(),
            name,
            base_model,
            num_images,
            total_steps,
            output_dir,
            backend,
            created_at: current_timestamp(),
            completed_at: None,
            error: None,
        }
    }

    /// Mark job as completed
    pub fn mark_completed(&mut self) {
        self.completed_at = Some(current_timestamp());
    }

    /// Mark job as failed with error
    pub fn mark_failed(&mut self, error: String) {
        self.completed_at = Some(current_timestamp());
        self.error = Some(error);
    }

    /// Get job duration in seconds
    pub fn duration_secs(&self) -> i64 {
        match self.completed_at {
            Some(end) => end - self.created_at,
            None => current_timestamp() - self.created_at,
        }
    }
}

/// Job store for managing training jobs
#[derive(Debug)]
pub struct JobStore {
    jobs: std::collections::HashMap<JobId, TrainingJob>,
    progress: std::collections::HashMap<JobId, TrainingProgress>,
}

impl JobStore {
    /// Create new job store
    pub fn new() -> Self {
        Self {
            jobs: std::collections::HashMap::new(),
            progress: std::collections::HashMap::new(),
        }
    }

    /// Add new job
    pub fn add_job(&mut self, job: TrainingJob) -> JobId {
        let id = job.id.clone();
        let total_steps = job.total_steps;

        self.jobs.insert(id.clone(), job);
        self.progress.insert(id.clone(), TrainingProgress::new(id.clone(), total_steps));

        id
    }

    /// Get job by ID
    pub fn get_job(&self, id: &JobId) -> Option<&TrainingJob> {
        self.jobs.get(id)
    }

    /// Get progress by ID
    pub fn get_progress(&self, id: &JobId) -> Option<&TrainingProgress> {
        self.progress.get(id)
    }

    /// Update progress
    pub fn update_progress(
        &mut self,
        id: &JobId,
        current_step: u64,
        loss: Option<f32>,
    ) -> Result<()> {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.update(current_step, loss);
            Ok(())
        } else {
            anyhow::bail!("Job not found: {:?}", id)
        }
    }

    /// Mark job as started
    pub fn start_job(&mut self, id: &JobId) -> Result<()> {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.started();
        }
        if self.jobs.contains_key(id) {
            Ok(())
        } else {
            anyhow::bail!("Job not found: {:?}", id)
        }
    }

    /// Mark job as completed
    pub fn complete_job(&mut self, id: &JobId) -> Result<()> {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.completed();
        }
        if let Some(job) = self.jobs.get_mut(id) {
            job.mark_completed();
            Ok(())
        } else {
            anyhow::bail!("Job not found: {:?}", id)
        }
    }

    /// Mark job as failed
    pub fn fail_job(&mut self, id: &JobId, error: String) -> Result<()> {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.failed();
        }
        if let Some(job) = self.jobs.get_mut(id) {
            job.mark_failed(error);
            Ok(())
        } else {
            anyhow::bail!("Job not found: {:?}", id)
        }
    }

    /// List all jobs
    pub fn list_jobs(&self) -> Vec<&TrainingJob> {
        self.jobs.values().collect()
    }

    /// List jobs by status
    pub fn list_jobs_by_status(&self, status: JobStatus) -> Vec<JobId> {
        self.progress
            .iter()
            .filter(|(_, p)| p.status == status)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Save jobs to JSON file
    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        let data = serde_json::json!({
            "jobs": self.jobs.values().collect::<Vec<_>>(),
            "progress": self.progress.values().collect::<Vec<_>>(),
        });
        std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
        Ok(())
    }
}

impl Default for JobStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_id_generation() {
        let id1 = JobId::new();
        let id2 = JobId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_training_progress() {
        let job_id = JobId::new();
        let mut progress = TrainingProgress::new(job_id.clone(), 100);

        assert_eq!(progress.current_step, 0);
        assert_eq!(progress.progress, 0.0);

        progress.update(50, Some(0.5));
        assert_eq!(progress.current_step, 50);
        assert!(progress.progress > 49.0 && progress.progress < 51.0);
        assert_eq!(progress.current_loss, Some(0.5));
    }

    #[test]
    fn test_training_job() {
        let job = TrainingJob::new(
            "test-job".to_string(),
            "stable-diffusion-xl".to_string(),
            20,
            500,
            PathBuf::from("/tmp/lora"),
            "comfyui".to_string(),
        );

        assert_eq!(job.base_model, "stable-diffusion-xl");
        assert_eq!(job.num_images, 20);
        assert!(job.completed_at.is_none());
    }

    #[test]
    fn test_job_store() {
        let mut store = JobStore::new();
        let job = TrainingJob::new(
            "test-job".to_string(),
            "stable-diffusion-xl".to_string(),
            20,
            100,
            PathBuf::from("/tmp/lora"),
            "comfyui".to_string(),
        );

        let job_id = job.id.clone();
        store.add_job(job);

        assert!(store.get_job(&job_id).is_some());
        assert!(store.get_progress(&job_id).is_some());

        store.start_job(&job_id).unwrap();
        let progress = store.get_progress(&job_id).unwrap();
        assert_eq!(progress.status, JobStatus::Running);
    }
}
