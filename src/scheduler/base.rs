use std::time::{SystemTime, Duration};
use rand::Rng;
use chrono::Utc;

use crate::error::Error as JobSchedulerError;
use crate::job::{JobBuilder, JobExecutor};
use crate::scheduler::types::{Schedule, ScheduleType, RecurringSchedule, RecurringInterval, RandomSchedule};

pub trait SchedulerRunner {
    fn add_job(&mut self, job: JobBuilder) -> Result<(), JobSchedulerError>;
    fn run_pending(&mut self) -> Result<(), JobSchedulerError>;
    /// Return the next scheduled run time among all jobs (system time).
    fn next_run(&self) -> Option<SystemTime>;
    fn list_all_jobs(&self) -> Vec<&JobBuilder>;
}

pub struct Scheduler {
    jobs: Vec<JobBuilder>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    fn compute_next_run(schedule: &mut Schedule) -> Option<SystemTime> {
        if let Some(max_runs) = schedule.max_runs {
            if schedule.run_count >= max_runs {
                return None;
            }
        }

        match &mut schedule.schedule_type {
            ScheduleType::Once(_time) => None, // Runs once, no next run
            ScheduleType::Random(_) => None, // Runs once at the pre-calculated time, no next run
            ScheduleType::Recurring(recurring) => {
                // calculate delta based on interval
                let delta = match &recurring.interval {
                    RecurringInterval::Secondly(opt) => {
                        Duration::from_secs(opt.unwrap_or(1).into())
                    },
                    RecurringInterval::Hourly(opt) => {
                        Duration::from_secs(3600 * u64::from(opt.unwrap_or(1)))
                    },
                    RecurringInterval::Daily(opt) => {
                        Duration::from_secs(86400 * u64::from(opt.unwrap_or(1)))
                    },
                    RecurringInterval::Weekly(opt) => {
                        Duration::from_secs(7 * 86400 * u64::from(opt.unwrap_or(1)))
                    },
                    RecurringInterval::Monthly(opt) => {
                        Duration::from_secs(30 * 86400 * u64::from(opt.unwrap_or(1)))
                    },
                    RecurringInterval::Custom { expression, frequency } => {
                        let days = match expression.as_str() {
                            "daily" => 1,
                            "weekly" => 7,
                            "monthly" => 30,
                            _ => frequency.unwrap_or(1),
                        };
                        Duration::from_secs(days as u64 * 86400)
                    },
                };
                // update next_run
                let next = recurring.next_run + delta;
                recurring.next_run = next;
                Some(next)
            }
            ScheduleType::Cron(cron_schedule) => {
                let now = Utc::now();
                cron_schedule.upcoming(Utc).next().map(|dt| dt.into())
            }
        }
    }

    // Helper to peek next run for a schedule without mutating it
    fn peek_next_run(schedule: &Schedule) -> Option<SystemTime> {
        // respect max_runs
        if let Some(max) = schedule.max_runs {
            if schedule.run_count >= max {
                return None;
            }
        }
        match &schedule.schedule_type {
            ScheduleType::Once(_) => None,
            ScheduleType::Random(_) => None,
            ScheduleType::Recurring(rec) => Some(rec.next_run),
            ScheduleType::Cron(cron_schedule) => cron_schedule.upcoming(Utc).next().map(|dt| dt.into()),
        }
    }
}

impl SchedulerRunner for Scheduler {
    fn add_job(&mut self, job: JobBuilder) -> Result<(), JobSchedulerError> {
        if job.schedules.is_empty() {
            return Err(JobSchedulerError::MissingSchedule);
        }
        if job.handler.is_none() {
            return Err(JobSchedulerError::HandlerNotBuilt);
        }
        self.jobs.push(job);
        Ok(())
    }

    fn run_pending(&mut self) -> Result<(), JobSchedulerError> {
        let now = SystemTime::now();
        for job in self.jobs.iter_mut() {
            if let Some(next) = job.next_run {
                if next <= now {
                    job.run()?;
                    job.last_run = Some(now);
                    // update each schedule that fired
                    for sched in job.schedules.iter_mut() {
                        if let Some(rn) = Scheduler::peek_next_run(sched) {
                            if rn <= now {
                                sched.run_count += 1;
                                Scheduler::compute_next_run(sched);
                            }
                        }
                    }
                    // recompute earliest next_run across schedules
                    job.next_run = job.schedules.iter()
                        .filter_map(|s| Scheduler::peek_next_run(s))
                        .min();
                }
            }
        }
        Ok(())
    }

    fn next_run(&self) -> Option<SystemTime> {
        self.jobs.iter().filter_map(|job| job.next_run).min()
    }

    fn list_all_jobs(&self) -> Vec<&JobBuilder> {
        self.jobs.iter().collect()
    }
}
