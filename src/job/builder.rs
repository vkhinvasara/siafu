use anyhow::Error;
use chrono::{DateTime, Utc};
use crate::types::Schedule;
use uuid::Uuid;
use crate::error::Error as JobSchedulerError;
use super::JobExecutor;

pub struct JobBuilder {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub schedule: Option<Schedule>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub handler: Option<fn() -> anyhow::Result<()>>,
}

impl JobBuilder {
    pub fn new(name: Option<&str>, description: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.map(|n| n.to_string()),
            description: description.map(|n| n.to_string()),
            schedule: None,
            last_run: None,
            next_run: None,
            handler: None,
        }
    }

    pub fn schedule_job(mut self, schedule: Schedule) -> Self {
        self.schedule = Some(schedule);
        self
    }

    pub fn add_handler(mut self, handler: fn() -> anyhow::Result<()>) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn build(self) -> JobBuilder {
        JobBuilder {
            id: self.id,
            name: self.name,
            description: self.description,
            schedule: self.schedule,
            last_run: self.last_run,
            next_run: self.next_run,
            handler: self.handler,
        }
    }
}

impl JobExecutor for JobBuilder {
    fn run(&mut self) -> Result<(), JobSchedulerError> {
        let handler = self.handler.ok_or_else(|| JobSchedulerError::HandlerNotBuilt)?;
        handler().map_err(|e| JobSchedulerError::ExecutionFailed(e.to_string()))
    }
}

