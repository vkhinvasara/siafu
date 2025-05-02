use chrono::{DateTime, Utc};
use crate::error::Error as JobSchedulerError;
use crate::job::{JobBuilder, JobExecutor};
use crate::scheduler::types::{Schedule, ScheduleType, RecurringSchedule, RecurringInterval, RandomSchedule};

pub trait Scheduler {
    fn add_job(&mut self, job: JobBuilder) -> Result<(), JobSchedulerError>;
    fn run_pending(&mut self) -> Result<(), JobSchedulerError>;
    fn next_run(&self) -> Option<DateTime<Utc>>;
    fn list_all_jobs(&self) -> Vec<&JobBuilder>;
}

pub struct BasicScheduler {
    jobs: Vec<JobBuilder>,
}

impl BasicScheduler {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    fn compute_next_run(schedule: &mut Schedule) -> Option<DateTime<Utc>> {
        if let Some(max_runs) = schedule.max_runs {
            if schedule.run_count >= max_runs {
                return None;
            }
        }
        
        match &mut schedule.schedule_type {
            ScheduleType::Once(_time) => {
                None
            },
            ScheduleType::Recurring(recurring) => {
                let new_next_run = match &recurring.interval {
                    RecurringInterval::Secondly(opt) => {
                        recurring.next_run + chrono::Duration::seconds(opt.unwrap_or(1) as i64)
                    },
                    RecurringInterval::Hourly(opt) => {
                        recurring.next_run + chrono::Duration::hours(opt.unwrap_or(1) as i64)
                    },
                    RecurringInterval::Daily(opt) => {
                        recurring.next_run + chrono::Duration::days(opt.unwrap_or(1) as i64)
                    },
                    RecurringInterval::Weekly(opt) => {
                        recurring.next_run + chrono::Duration::days(7 * opt.unwrap_or(1) as i64)
                    },
                    RecurringInterval::Monthly(opt) => {
                        recurring.next_run + chrono::Duration::days(30 * opt.unwrap_or(1) as i64)
                    },
                    RecurringInterval::Custom { expression, frequency } => {
                        let additional_days = match expression.as_str() {
                            "daily" => 1,
                            "weekly" => 7,
                            "monthly" => 30,
                            _ => frequency.unwrap_or(1),
                        };
                        recurring.next_run + chrono::Duration::days(additional_days as i64)
                    },
                };
                recurring.next_run = new_next_run;
                Some(new_next_run)
            },
            ScheduleType::Random(random) => {
                let duration = random.end_time.signed_duration_since(random.start_time);
                use rand::Rng;
                let mut rng = rand::rng();
                let secs = duration.num_seconds();
                let random_secs = rng.random_range(0..=secs);
                Some(random.start_time + chrono::Duration::seconds(random_secs))
            },
        }
    }
}

impl Scheduler for BasicScheduler {
    fn add_job(&mut self, job: JobBuilder) -> Result<(), JobSchedulerError> {
        if job.schedule.is_none() {
            return Err(JobSchedulerError::MissingSchedule);
        }
        if job.handler.is_none() {
            return Err(JobSchedulerError::HandlerNotBuilt);
        }
        self.jobs.push(job);
        Ok(())
    }

    fn run_pending(&mut self) -> Result<(), JobSchedulerError> {
        let now = Utc::now();
        for job in self.jobs.iter_mut() {
            if let Some(next_run) = job.next_run {
                if next_run <= now {
                    job.run()?;
                    if let Some(schedule) = job.schedule.as_mut() {
                        schedule.run_count += 1;
                        job.last_run = Some(now);
                        job.next_run = BasicScheduler::compute_next_run(schedule);
                    }
                }
            }
        }
        Ok(())
    }

    fn next_run(&self) -> Option<DateTime<Utc>> {
        self.jobs
            .iter()
            .filter_map(|job| job.next_run)
            .min()
    }

    fn list_all_jobs(&self) -> Vec<&JobBuilder> {
        self.jobs.iter().collect()
    }
}
