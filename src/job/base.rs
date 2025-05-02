use std::time::SystemTime;
use crate::error::Error as JobSchedulerError;

pub trait JobExecutor {
    
    fn run(&mut self) -> Result<(), JobSchedulerError>;

    fn get_next_run(&self) -> Option<SystemTime> {
        None
    }
}
