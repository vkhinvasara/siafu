//! JobBuilder provides a fluent API to configure scheduled jobs with various types (once, recurring, cron, random),
//! set maximum repeats, and assign execution handlers.
//!
//! # Examples
//!
//! ```rust
//! use siafu::{JobBuilder, ScheduleTime};
//! use siafu::scheduler::types::RecurringInterval;
//! use std::time::{Duration, SystemTime};
//!
//! // One-time job after 5 seconds
//! let job1 = JobBuilder::new("once-job")
//!     .once(ScheduleTime::Delay(Duration::from_secs(5)))
//!     .add_handler(|| println!("Run once"))
//!     .build();
//!
//! // Recurring job every minute, up to 10 times
//! let job2 = JobBuilder::new("recurring-job")
//!     .recurring(RecurringInterval::Minutely(1), None)
//!     .max_repeat(10)
//!     .add_handler(|| println!("Recurring run"))
//!     .build();
//!
//! // Cron job: every hour on the hour
//! let job3 = JobBuilder::new("cron-job")
//!     .cron("0 0 * * * * *")
//!     .add_handler(|| println!("Hourly cron"))
//!     .build();
//! ```

use std::time::{SystemTime, Duration};
use crate::scheduler::types::{Schedule, ScheduleType, RandomSchedule, RecurringSchedule, RecurringInterval};
use uuid::Uuid;
use crate::error::Error as JobSchedulerError;
use super::JobExecutor;
use chrono::Utc;
use rand::{rng, Rng};
use cron::Schedule as CronSchedule;
use crate::utils::time::ScheduleTime;
use std::str::FromStr;

// Define the handler type alias
type JobHandler = Box<dyn Fn() + Send + 'static>;

pub struct JobBuilder {
    pub id: Uuid,
    pub name: Option<String>,
    pub schedules: Vec<Schedule>,
    pub last_run: Option<SystemTime>,
    pub next_run: Option<SystemTime>,
    pub handler: Option<JobHandler>,
}

impl JobBuilder {
    /// Construct a new JobBuilder with optional name.
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: if name.is_empty() { None } else { Some(name.to_string()) },
            schedules: Vec::new(),
            last_run: None,
            next_run: None,
            handler: None,
        }
    }

    /// Schedule the job to run once at the specified time.
    /// 
    /// Takes a ScheduleTime which can be either a specific time (At) or a delay (Delay).
    pub fn once(mut self, time: ScheduleTime) -> Self {
        match time {
            ScheduleTime::At(system_time) => {
                let sched = Schedule { schedule_type: ScheduleType::Once(system_time), max_runs: Some(1), run_count: 0 };
                self.next_run = self.next_run.map_or(Some(system_time), |nr| Some(nr.min(system_time)));
                self.schedules.push(sched);
            },
            ScheduleTime::Delay(duration) => {
                let system_time = SystemTime::now() + duration;
                let sched = Schedule { schedule_type: ScheduleType::Once(system_time), max_runs: Some(1), run_count: 0 };
                self.next_run = self.next_run.map_or(Some(system_time), |nr| Some(nr.min(system_time)));
                self.schedules.push(sched);
            }
        }
        self
    }

    /// Schedule the job with a recurring interval.
    ///
    /// This method takes a RecurringInterval directly and an optional start time.
    pub fn recurring(mut self, interval: RecurringInterval, start_time: Option<ScheduleTime>) -> Self {
        // Determine the first run time
        let first_run = match start_time {
            Some(ScheduleTime::At(time)) => time,
            Some(ScheduleTime::Delay(delay)) => SystemTime::now() + delay,
            None => {
                // Default to a reasonable start time based on the interval type
                let now = SystemTime::now();
                match &interval {
                    RecurringInterval::Secondly(secs) => now + Duration::from_secs(*secs as u64),
                    RecurringInterval::Minutely(mins) => now + Duration::from_secs(*mins as u64 * 60),
                    RecurringInterval::Hourly(hours) => now + Duration::from_secs(*hours as u64 * 3600),
                    RecurringInterval::Daily(days) => now + Duration::from_secs(*days as u64 * 86400),
                    RecurringInterval::Weekly(weeks) => now + Duration::from_secs(*weeks as u64 * 604800),
                    RecurringInterval::Monthly(months) => now + Duration::from_secs(*months as u64 * 2592000), // Approximate 30 days
                    RecurringInterval::Custom { .. } => now + Duration::from_secs(60), // Default to 1 minute
                }
            }
        };
        
        // Create the recurring schedule
        let recurring = RecurringSchedule {
            interval,
            next_run: first_run,
        };
        
        // Add to schedules
        let sched = Schedule { schedule_type: ScheduleType::Recurring(recurring.clone()), max_runs: None, run_count: 0 };
        self.next_run = self.next_run.map_or(Some(first_run), |nr| Some(nr.min(first_run)));
        self.schedules.push(sched);
        self
    }

    // Keep the every method for backward compatibility or convenience
    /// Schedule the job with a recurring interval using a standard Duration.
    /// 
    /// This is a convenience method that converts a Duration to an appropriate RecurringInterval.
    pub fn every(self, interval: Duration, start_time: Option<ScheduleTime>) -> Self {
        let recurring_interval = duration_to_recurring_interval(interval);
        self.recurring(recurring_interval, start_time)
    }

    /// Schedule the job using a cron expression.
    pub fn cron(mut self, cron_schedule: &str) -> Self {
        // Try to parse the cron expression
        match CronSchedule::from_str(cron_schedule) {
            Ok(schedule) => {
                if let Some(rt) = schedule.upcoming(Utc).next().map(|dt| dt.into()) {
                    self.next_run = self.next_run.map_or(Some(rt), |nr| Some(nr.min(rt)));
                }
                let sched = Schedule { 
                    schedule_type: ScheduleType::Cron(schedule.clone()), 
                    max_runs: None, 
                    run_count: 0 
                };
                self.schedules.push(sched);
            },
            Err(_) => {
                // In case of an error, don't add this schedule
                // Could also return a Result instead
            }
        }
        self
    }

    /// Schedule the job at a random time between start_time and end_time.
    pub fn random(mut self, start: ScheduleTime, end: ScheduleTime) -> Self {
        // Convert both times to SystemTime
        let start_time = match start {
            ScheduleTime::At(time) => time,
            ScheduleTime::Delay(delay) => SystemTime::now() + delay,
        };
        
        let end_time = match end {
            ScheduleTime::At(time) => time,
            ScheduleTime::Delay(delay) => SystemTime::now() + delay,
        };
        
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
        
        let sched = Schedule { 
            schedule_type: ScheduleType::Random(rand_sched), 
            max_runs: None, 
            run_count: 0 
        };
        
        self.schedules.push(sched);
        self
    }

    /// Limit the number of times a scheduled job will run.
    pub fn max_repeat(mut self, max_runs: u32) -> Self {
        if let Some(last) = self.schedules.last_mut() {
            last.max_runs = Some(max_runs);
        }
        self
    }

    /// Assign a handler to the job. Accepts a closure that takes no arguments and returns nothing.
    pub fn add_handler<F>(mut self, handler: F) -> Self 
    where F: Fn() + Send + 'static {
        self.handler = Some(Box::new(handler));
        self
    }

    /// Finalize the builder.
    pub fn build(self) -> JobBuilder {
        JobBuilder { ..self }
    }
}

impl JobExecutor for JobBuilder {
    fn run(&mut self) -> Result<(), JobSchedulerError> {
        if let Some(handler) = &self.handler {
            handler();
            Ok(())
        } else {
            Err(JobSchedulerError::HandlerNotBuilt)
        }
    }
}

// Helper function for backward compatibility with the every method
fn duration_to_recurring_interval(duration: Duration) -> RecurringInterval {
    let secs = duration.as_secs();
    
    if secs % 86400 == 0 && secs > 0 {
        // Daily (86400 seconds in a day)
        RecurringInterval::Daily((secs / 86400) as u32)
    } else if secs % 3600 == 0 && secs > 0 {
        // Hourly (3600 seconds in an hour)
        RecurringInterval::Hourly((secs / 3600) as u32)
    } else if secs % 60 == 0 && secs > 0 {
        // Minutely (60 seconds in a minute)
        RecurringInterval::Minutely((secs / 60) as u32)
    } else {
        // Secondly
        RecurringInterval::Secondly(secs as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, Duration};


    #[test]
    fn test_schedule_job_once() {
        let job_builder = JobBuilder::new("test_once");
        let target_time = ScheduleTime::At(SystemTime::now() + Duration::from_secs(5));
        let scheduled_job = job_builder.once(target_time.clone());

        assert!(!scheduled_job.schedules.is_empty());
        let schedule_in_job = &scheduled_job.schedules[0];
        assert!(matches!(schedule_in_job.schedule_type, ScheduleType::Once(_)));
        assert_eq!(schedule_in_job.max_runs, Some(1));
        assert!(scheduled_job.next_run.is_some());
        let diff = if let ScheduleTime::At(target_time) = target_time {
            let next_run = scheduled_job.next_run.unwrap();
            if target_time > next_run {
                target_time.duration_since(next_run).ok()
            } else {
                next_run.duration_since(target_time).ok()
            }
        } else {
            None
        };
        assert!(diff.unwrap_or_default() < Duration::from_millis(100));
    }

    #[test]
    fn test_schedule_job_cron() {
        let job_builder = JobBuilder::new("test_cron");
        let cron_schedule_expr = "* * * * * * *";
        let scheduled_job = job_builder.cron(cron_schedule_expr);

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
        let job_builder = JobBuilder::new("test_recurring");
        let interval = Duration::from_secs(10);
        let first_run = ScheduleTime::At(SystemTime::now() + interval);
        let scheduled_job = job_builder.every(interval, Some(first_run.clone()));

        assert!(!scheduled_job.schedules.is_empty());
        let schedule_in_job = &scheduled_job.schedules[0];
        assert!(matches!(schedule_in_job.schedule_type, ScheduleType::Recurring(_)));
        assert_eq!(schedule_in_job.max_runs, None);
        assert!(scheduled_job.next_run.is_some());
        let diff = if let ScheduleTime::At(first_run) = first_run {
            let next_run = scheduled_job.next_run.unwrap();
            if first_run > next_run {
                first_run.duration_since(next_run).ok()
            } else {
                next_run.duration_since(first_run).ok()
            }
        } else {
            None
        };
        assert!(diff.unwrap_or_default() < Duration::from_millis(100));
    }

    #[test]
    fn test_schedule_job_random() {
        let job_builder = JobBuilder::new("test_random");
        let start_time = ScheduleTime::At(SystemTime::now() + Duration::from_secs(1));
        let end_time = ScheduleTime::At(SystemTime::now() + Duration::from_secs(10));
        let scheduled_job = job_builder.random(start_time.clone(), end_time.clone());
        let next_run = scheduled_job.next_run.unwrap();
        if let ScheduleTime::At(start_time) = start_time {
            if let ScheduleTime::At(end_time) = end_time {
                assert!(next_run >= start_time && next_run < end_time);
            }
        }
    }

    #[test]
    fn test_schedule_job_random_invalid_range() {
        let job_builder = JobBuilder::new("test_random_invalid");
        let start_time = ScheduleTime::At(SystemTime::now() + Duration::from_secs(10));
        let end_time = ScheduleTime::At(SystemTime::now() - Duration::from_secs(1));
        let scheduled_job = job_builder.random(start_time, end_time);
        assert!(scheduled_job.next_run.is_none());
    }

    #[test]
    fn test_schedule_job_recurring_direct() {
        let start_time = Some(ScheduleTime::At(SystemTime::now() + Duration::from_secs(5)));
        
        // Create jobs with different interval types
        let hourly_job = JobBuilder::new("test_direct_recurring").recurring(RecurringInterval::Hourly(2), start_time.clone());
        let daily_job = JobBuilder::new("test_direct_recurring").recurring(RecurringInterval::Daily(1), start_time.clone());
        let weekly_job = JobBuilder::new("test_direct_recurring").recurring(RecurringInterval::Weekly(3), start_time.clone());

        // Verify hourly job
        assert!(!hourly_job.schedules.is_empty());
        if let ScheduleType::Recurring(rec) = &hourly_job.schedules[0].schedule_type {
            assert_eq!(rec.interval, RecurringInterval::Hourly(2));
        } else {
            panic!("Expected Recurring schedule type");
        }

        // Verify daily job
        assert!(!daily_job.schedules.is_empty());
        if let ScheduleType::Recurring(rec) = &daily_job.schedules[0].schedule_type {
            assert_eq!(rec.interval, RecurringInterval::Daily(1));
        } else {
            panic!("Expected Recurring schedule type");
        }

        // Verify weekly job
        assert!(!weekly_job.schedules.is_empty());
        if let ScheduleType::Recurring(rec) = &weekly_job.schedules[0].schedule_type {
            assert_eq!(rec.interval, RecurringInterval::Weekly(3));
        } else {
            panic!("Expected Recurring schedule type");
        }

        // Test without explicit start time (should use default)
        let minutely_job = JobBuilder::new("test_direct_recurring").recurring(RecurringInterval::Minutely(5), None);
        assert!(!minutely_job.schedules.is_empty());
        assert!(minutely_job.next_run.is_some());
        let now = SystemTime::now();
        assert!(minutely_job.next_run.unwrap() > now);
    }
}

