//! JobExecutor defines the interface for executing scheduled jobs.
//!
//! Implementors should run the job logic in `run` and may override `get_next_run`

use std::time::SystemTime;
use crate::error::Error as JobSchedulerError;

/// JobExecutor trait for executing scheduled jobs.
pub trait JobExecutor {
    /// Execute the job's handler.
    ///
    /// Returns `Ok(())` on success, or an `Error` if execution fails or handler is missing.
    fn run(&mut self) -> Result<(), JobSchedulerError>;

    /// Optionally return the next scheduled run time for this job.
    ///
    /// Default implementation returns `None`.
    fn get_next_run(&self) -> Option<SystemTime> {
        None
    }
}
