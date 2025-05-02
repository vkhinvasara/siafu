use std::time::{SystemTime, Duration};
use crate::scheduler::types::{Schedule, ScheduleType, RandomSchedule, RecurringSchedule};
use uuid::Uuid;
use crate::error::Error as JobSchedulerError;
use super::JobExecutor;
use chrono::Utc;
use rand::{rng, Rng};
use cron::Schedule as CronSchedule;

pub struct JobBuilder {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub schedules: Vec<Schedule>,
    pub last_run: Option<SystemTime>,
    pub next_run: Option<SystemTime>,
    pub handler: Option<fn() -> anyhow::Result<()>>,
}

impl JobBuilder {
    /// Construct a new JobBuilder with optional name and description.
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: if name.is_empty() { None } else { Some(name.to_string()) },
            description: if description.is_empty() { None } else { Some(description.to_string()) },
            schedules: Vec::new(),
            last_run: None,
            next_run: None,
            handler: None,
        }
    }

    /// Schedule the job to run once at the specified time.
    pub fn once(mut self, time: SystemTime) -> Self {
        let sched = Schedule { schedule_type: ScheduleType::Once(time), max_runs: Some(1), run_count: 0 };
        self.next_run = self.next_run.map_or(Some(time), |nr| Some(nr.min(time)));
        self.schedules.push(sched);
        self
    }

    /// Schedule the job with a recurring interval.
    pub fn recurring(mut self, recurring: RecurringSchedule) -> Self {
        let first_run = recurring.next_run;
        let sched = Schedule { schedule_type: ScheduleType::Recurring(recurring.clone()), max_runs: None, run_count: 0 };
        self.next_run = self.next_run.map_or(Some(first_run), |nr| Some(nr.min(first_run)));
        self.schedules.push(sched);
        self
    }

    /// Schedule the job using a cron expression.
    pub fn cron(mut self, cron_schedule: CronSchedule) -> Self {
        if let Some(rt) = cron_schedule.upcoming(Utc).next().map(|dt| dt.into()) {
            self.next_run = self.next_run.map_or(Some(rt), |nr| Some(nr.min(rt)));
        }
        let sched = Schedule { schedule_type: ScheduleType::Cron(cron_schedule.clone()), max_runs: None, run_count: 0 };
        self.schedules.push(sched);
        self
    }

    /// Schedule the job at a random time between start_time and end_time.
    pub fn random(mut self, start_time: SystemTime, end_time: SystemTime) -> Self {
        let rand_sched = RandomSchedule { start_time, end_time };
        let rt = if end_time > start_time {
            let range = end_time.duration_since(start_time).unwrap();
            let mut rng = rng();
            let nanos = range.as_nanos() as u64;
            let offset = rng.random_range(0..nanos);
            Some(start_time + Duration::from_nanos(offset))
        } else { None };
        if let Some(rn) = rt {
            self.next_run = self.next_run.map_or(Some(rn), |nr| Some(nr.min(rn)));
        }
        let sched = Schedule { schedule_type: ScheduleType::Random(rand_sched), max_runs: None, run_count: 0 };
        self.schedules.push(sched);
        self
    }

    /// Limit the number of times a scheduled job will run.
    pub fn repeat(mut self, max_runs: u32) -> Self {
        if let Some(last) = self.schedules.last_mut() {
            last.max_runs = Some(max_runs);
        }
        self
    }

    /// Assign a handler to the job.
    pub fn add_handler(mut self, handler: fn() -> anyhow::Result<()>) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Finalize the builder.
    pub fn build(self) -> JobBuilder {
        JobBuilder { ..self }
    }
}

impl JobExecutor for JobBuilder {
    fn run(&mut self) -> Result<(), JobSchedulerError> {
        let handler = self.handler.ok_or_else(|| JobSchedulerError::HandlerNotBuilt)?;
        handler().map_err(|e| JobSchedulerError::ExecutionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::types::{ScheduleType, RecurringSchedule, RecurringInterval};
    use std::time::{SystemTime, Duration};
    use std::str::FromStr;

    fn dummy_handler() -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn test_schedule_job_once() {
        let job_builder = JobBuilder::new("test_once", "");
        let target_time = SystemTime::now() + Duration::from_secs(5);
        let scheduled_job = job_builder.once(target_time);

        assert!(!scheduled_job.schedules.is_empty());
        let schedule_in_job = &scheduled_job.schedules[0];
        assert!(matches!(schedule_in_job.schedule_type, ScheduleType::Once(_)));
        assert_eq!(schedule_in_job.max_runs, Some(1));
        assert!(scheduled_job.next_run.is_some());
        let diff = if target_time > scheduled_job.next_run.unwrap() {
            target_time.duration_since(scheduled_job.next_run.unwrap())
        } else {
            scheduled_job.next_run.unwrap().duration_since(target_time)
        };
        assert!(diff.unwrap_or_default() < Duration::from_millis(100));
    }

    #[test]
    fn test_schedule_job_cron() {
        let job_builder = JobBuilder::new("test_cron", "");
        let cron_schedule_expr = "* * * * * * *";
        let cron_schedule = CronSchedule::from_str(cron_schedule_expr).unwrap();
        let scheduled_job = job_builder.cron(cron_schedule);

        assert!(!scheduled_job.schedules.is_empty());
        let schedule_in_job = &scheduled_job.schedules[0];
        assert!(matches!(schedule_in_job.schedule_type, ScheduleType::Cron(_)));
        assert_eq!(schedule_in_job.max_runs, None);
        assert!(scheduled_job.next_run.is_some());

        let now = SystemTime::now();
        let next_run = scheduled_job.next_run.unwrap();
        assert!(next_run >= now);
        assert!(next_run.duration_since(now).unwrap_or_default() < Duration::from_secs(2));
    }

    #[test]
    fn test_schedule_job_recurring() {
        let job_builder = JobBuilder::new("test_recurring", "");
        let interval = Duration::from_secs(10);
        let first_run = SystemTime::now() + interval;
        let recurring_schedule = RecurringSchedule {
            interval: RecurringInterval::Secondly(Some(10)),
            next_run: first_run,
        };
        let scheduled_job = job_builder.recurring(recurring_schedule);

        assert!(!scheduled_job.schedules.is_empty());
        let schedule_in_job = &scheduled_job.schedules[0];
        assert!(matches!(schedule_in_job.schedule_type, ScheduleType::Recurring(_)));
        assert_eq!(schedule_in_job.max_runs, None);
        assert!(scheduled_job.next_run.is_some());
        let diff = if first_run > scheduled_job.next_run.unwrap() {
            first_run.duration_since(scheduled_job.next_run.unwrap())
        } else {
            scheduled_job.next_run.unwrap().duration_since(first_run)
        };
        assert!(diff.unwrap_or_default() < Duration::from_millis(100));
    }

    #[test]
    fn test_schedule_job_random() {
        let job_builder = JobBuilder::new("test_random", "");
        let start_time = SystemTime::now() + Duration::from_secs(1);
        let end_time = start_time + Duration::from_secs(10);
        let scheduled_job = job_builder.random(start_time, end_time);
        let next_run = scheduled_job.next_run.unwrap();
        assert!(next_run >= start_time && next_run < end_time);
    }

    #[test]
    fn test_schedule_job_random_invalid_range() {
        let job_builder = JobBuilder::new("test_random_invalid", "");
        let start_time = SystemTime::now() + Duration::from_secs(10);
        let end_time = start_time - Duration::from_secs(1);
        let scheduled_job = job_builder.random(start_time, end_time);
        assert!(scheduled_job.next_run.is_none());
    }
}

