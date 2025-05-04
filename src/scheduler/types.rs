//! Defines scheduling types and metadata for job schedules in Siafu.
//!
//! - `ScheduleType`: Enumeration of scheduling patterns (once, recurring, random, cron).
//! - `Schedule`: Contains schedule metadata including max runs and run count.
//! - `RecurringSchedule` and `RandomSchedule`: Details for recurring and random patterns.
//! - `RecurringInterval`: Preset intervals or custom frequency values.
//!
//! # Examples
//!
//! ```rust
//! use siafu::scheduler::types::{Schedule, ScheduleType, RecurringInterval};
//! use siafu::utils::time::ScheduleTime;
//! use std::time::{SystemTime, Duration};
//! use cron::Schedule as CronSchedule;
//!
//! // One-time schedule at a specific SystemTime
//! let t = SystemTime::now() + Duration::from_secs(10);
//! let once = Schedule { schedule_type: ScheduleType::Once(t), max_runs: Some(1), run_count: 0 };
//!
//! // Recurring schedule every 5 seconds
//! let recur = Schedule {
//!     schedule_type: ScheduleType::Recurring(
//!         RecurringSchedule { interval: RecurringInterval::Secondly(5), next_run: t }
//!     ),
//!     max_runs: None,
//!     run_count: 0,
//! };
//!
//! // Cron schedule: every hour on the hour
//! let cron_expr = "0 0 * * * * *";
//! let cron_schedule = CronSchedule::from_str(cron_expr).unwrap();
//! let cron = Schedule { schedule_type: ScheduleType::Cron(cron_schedule), max_runs: None, run_count: 0 };
//! ```

use std::time::SystemTime;
use cron::Schedule as CronSchedule;

pub enum ScheduleType {
    Once(SystemTime),
    Recurring(RecurringSchedule),
    Random(RandomSchedule),
    Cron(CronSchedule),
}

pub struct Schedule {
    pub schedule_type: ScheduleType,
    pub max_runs: Option<u32>,
    pub run_count: u32,
}

#[derive(Clone)]
pub struct RecurringSchedule {
    pub interval: RecurringInterval,
    pub next_run: SystemTime,
}

pub struct RandomSchedule {
    pub start_time: SystemTime,
    pub end_time: SystemTime,
}

#[derive(Debug,Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecurringInterval {
    Secondly(u32), 
    Minutely(u32),
    Hourly(u32),   
    Daily(u32),    
    Weekly(u32),   
    Monthly(u32),  
    Custom { 
        expression: String, 
        frequency: u32, 
    },
}