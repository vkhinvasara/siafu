use std::time::{SystemTime, Duration};
use chrono::Utc;

use crate::error::Error as JobSchedulerError;
use crate::job::{JobBuilder, JobExecutor};
use crate::scheduler::types::{Schedule, ScheduleType, RecurringInterval};

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

    /// Add a job to the scheduler.
    pub fn add_job(&mut self, job: JobBuilder) -> Result<(), JobSchedulerError> {
        if job.schedules.is_empty() {
            return Err(JobSchedulerError::MissingSchedule);
        }
        if job.handler.is_none() {
            return Err(JobSchedulerError::HandlerNotBuilt);
        }
        self.jobs.push(job);
        Ok(())
    }

    /// Run all jobs that are scheduled to run now or earlier.
    pub fn run_pending(&mut self) -> Result<(), JobSchedulerError> {
        let now = SystemTime::now();
        for job in self.jobs.iter_mut() {
            if let Some(next) = job.next_run {
                if next <= now {
                    job.run()?;
                    job.last_run = Some(now);
                    // update each schedule that fired
                    for sched in job.schedules.iter_mut() {
                        if let Some(rn) = Self::peek_next_run(sched) {
                            if rn <= now {
                                sched.run_count += 1;
                                Self::compute_next_run(sched);
                            }
                        }
                    }
                    // recompute earliest next_run across schedules
                    job.next_run = job.schedules.iter()
                        .filter_map(|s| Self::peek_next_run(s))
                        .min();
                }
            }
        }
        Ok(())
    }

    /// Return the next scheduled run time among all jobs (system time).
    pub fn next_run(&self) -> Option<SystemTime> {
        self.jobs.iter().filter_map(|job| job.next_run).min()
    }

    /// List all jobs in the scheduler.
    pub fn list_all_jobs(&self) -> Vec<&JobBuilder> {
        // Return jobs sorted by next_run ascending, jobs with no next_run at the end
        let mut job_refs: Vec<&JobBuilder> = self.jobs.iter().collect();
        job_refs.sort_by(|a, b| match (a.next_run, b.next_run) {
            (Some(a_time), Some(b_time)) => a_time.cmp(&b_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        job_refs
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
                    RecurringInterval::Secondly(secs) => {
                        Duration::from_secs(*secs as u64)
                    },
                    RecurringInterval::Minutely(mins) => {
                        Duration::from_secs(60 * *mins as u64)
                    },
                    RecurringInterval::Hourly(hours) => {
                        Duration::from_secs(3600 * *hours as u64)
                    },
                    RecurringInterval::Daily(days) => {
                        Duration::from_secs(86400 * *days as u64)
                    },
                    RecurringInterval::Weekly(weeks) => {
                        Duration::from_secs(7 * 86400 * *weeks as u64)
                    },
                    RecurringInterval::Monthly(months) => {
                        Duration::from_secs(30 * 86400 * *months as u64)
                    },
                    RecurringInterval::Custom { expression, frequency } => {
                        let days = match expression.as_str() {
                            "daily" => 1,
                            "weekly" => 7,
                            "monthly" => 30,
                            _ => *frequency,
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
                // let now = Utc::now();
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

#[cfg(test)]
mod tests {
    use crate::scheduler::types::{RecurringSchedule, RecurringInterval};
    use crate::utils::time::ScheduleTime;

    use super::*;
    use std::thread::sleep;
    use cron::Schedule as CronSchedule;
    use std::str::FromStr;

    // Helper function for tests
    fn dummy_handler() -> anyhow::Result<()> {
        Ok(())
    }
    
    #[test]
    fn test_new_scheduler_empty() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.jobs.len(), 0);
        assert_eq!(scheduler.next_run(), None);
        assert_eq!(scheduler.list_all_jobs().len(), 0);
    }
    
    #[test]
    fn test_add_job() -> Result<(), JobSchedulerError> {
        let mut scheduler = Scheduler::new();
        let job = JobBuilder::new("test-job")
            .once(ScheduleTime::At(SystemTime::now() + Duration::from_secs(60)))
            .add_handler(dummy_handler)
            .build();
            
        scheduler.add_job(job)?;
        assert_eq!(scheduler.jobs.len(), 1);
        assert_eq!(scheduler.list_all_jobs().len(), 1);
        
        let job_ref = scheduler.list_all_jobs()[0];
        assert_eq!(job_ref.name, Some("test-job".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_add_job_no_schedule() {
        let mut scheduler = Scheduler::new();
        // Create a job without any schedules
        let job = JobBuilder::new("no-schedule")
            .add_handler(dummy_handler)
            .build();
            
        let result = scheduler.add_job(job);
        assert!(result.is_err());
        match result {
            Err(JobSchedulerError::MissingSchedule) => {},
            _ => panic!("Expected MissingSchedule error"),
        }
    }
    
    #[test]
    fn test_add_job_no_handler() {
        let mut scheduler = Scheduler::new();
        // Create a job without a handler
        let job = JobBuilder::new("no-handler")
            .once(ScheduleTime::At(SystemTime::now() + Duration::from_secs(60)))
            .build();
            
        let result = scheduler.add_job(job);
        assert!(result.is_err());
        match result {
            Err(JobSchedulerError::HandlerNotBuilt) => {},
            _ => panic!("Expected HandlerNotBuilt error"),
        }
    }
    
    #[test]
    fn test_next_run_single_job() -> Result<(), JobSchedulerError> {
        let mut scheduler = Scheduler::new();
        let target_time = SystemTime::now() + Duration::from_secs(60);
        
        let job = JobBuilder::new("test-job")
            .once(ScheduleTime::At(target_time))
            .add_handler(dummy_handler)
            .build();
            
        scheduler.add_job(job)?;
        
        let next = scheduler.next_run();
        assert!(next.is_some());
        
        let diff = if target_time > next.unwrap() {
            target_time.duration_since(next.unwrap())
        } else {
            next.unwrap().duration_since(target_time)
        };
        
        assert!(diff.unwrap_or_default() < Duration::from_millis(10));
        
        Ok(())
    }
    
    #[test]
    fn test_next_run_multiple_jobs() -> Result<(), JobSchedulerError> {
        let mut scheduler = Scheduler::new();
        let time1 = SystemTime::now() + Duration::from_secs(60);
        let time2 = SystemTime::now() + Duration::from_secs(30); // Earlier time
        
        let job1 = JobBuilder::new("job1")
            .once(ScheduleTime::At(time1))
            .add_handler(dummy_handler)
            .build();
            
        let job2 = JobBuilder::new("job2")
            .once(ScheduleTime::At(time2))
            .add_handler(dummy_handler)
            .build();
            
        scheduler.add_job(job1)?;
        scheduler.add_job(job2)?;
        
        // Should return the earlier of the two times (time2)
        let next = scheduler.next_run();
        assert!(next.is_some());
        
        let diff = if time2 > next.unwrap() {
            time2.duration_since(next.unwrap())
        } else {
            next.unwrap().duration_since(time2)
        };
        
        assert!(diff.unwrap_or_default() < Duration::from_millis(10));
        
        Ok(())
    }
    
    #[test]
    fn test_run_pending_job() -> Result<(), JobSchedulerError> {
        let mut scheduler = Scheduler::new();
        
        // Create a job that will run immediately
        let job = JobBuilder::new("immediate")
            .once(ScheduleTime::At(SystemTime::now()))
            .add_handler(dummy_handler)
            .build();
            
        scheduler.add_job(job)?;
        
        // Should be one job before running
        assert_eq!(scheduler.jobs.len(), 1);
        
        // Run pending jobs
        scheduler.run_pending()?;
        
        // Should still have one job but with updated last_run time and null next_run
        assert_eq!(scheduler.jobs.len(), 1);
        assert!(scheduler.jobs[0].last_run.is_some());
        assert!(scheduler.jobs[0].next_run.is_none()); // Should be None after running once
        
        Ok(())
    }
    
    #[test]
    fn test_run_recurring_jobs() -> Result<(), JobSchedulerError> {
        let mut scheduler = Scheduler::new();
        
        // Create a job that recurs every second
        let recur_time = SystemTime::now();
        
        let job = JobBuilder::new("recurring")
            .recurring(RecurringInterval::Secondly(1), Some(ScheduleTime::At(recur_time)))
            .add_handler(dummy_handler)
            .build();
            
        scheduler.add_job(job)?;
        
        // Initial state check
        assert_eq!(scheduler.jobs.len(), 1);
        assert!(scheduler.jobs[0].next_run.is_some());
        
        // Run pending jobs
        scheduler.run_pending()?;
        
        // Should have updated last_run and scheduled next run
        let last_run = scheduler.jobs[0].last_run;
        assert!(last_run.is_some());
        
        let next_run = scheduler.jobs[0].next_run;
        assert!(next_run.is_some());
        
        // Next run should be approximately one second after the initial time
        let expected_next = recur_time + Duration::from_secs(1);
        let diff = if expected_next > next_run.unwrap() {
            expected_next.duration_since(next_run.unwrap())
        } else {
            next_run.unwrap().duration_since(expected_next)
        };
        
        assert!(diff.unwrap_or_default() < Duration::from_millis(100));
        
        Ok(())
    }
    
    #[test]
    fn test_run_job_with_max_runs() -> Result<(), JobSchedulerError> {
        let mut scheduler = Scheduler::new();
        
        // Create a recurring job with max 2 runs
        let recur_time = SystemTime::now();
        
        let job = JobBuilder::new("limited-runs")
            .recurring(RecurringInterval::Secondly(1), Some(ScheduleTime::At(recur_time)))
            .repeat(2)
            .add_handler(dummy_handler)
            .build();
            
        scheduler.add_job(job)?;
        
        // Run first execution
        scheduler.run_pending()?;
        assert!(scheduler.jobs[0].next_run.is_some());
        
        // Wait a bit to ensure the next schedule is ready
        sleep(Duration::from_secs(1));
        
        // Run second execution
        scheduler.run_pending()?;
        
        // Wait a bit more
        sleep(Duration::from_secs(1));
        
        // Run again, but there should be no next run since we hit max_runs=2
        scheduler.run_pending()?;
        assert!(scheduler.jobs[0].next_run.is_none());
        
        Ok(())
    }
    
    #[test]
    fn test_list_all_jobs() -> Result<(), JobSchedulerError> {
        let mut scheduler = Scheduler::new();
        
        let job1 = JobBuilder::new("job1")
            .once(ScheduleTime::At(SystemTime::now() + Duration::from_secs(60)))
            .add_handler(dummy_handler)
            .build();
            
        let job2 = JobBuilder::new("job2")
            .once(ScheduleTime::At(SystemTime::now() + Duration::from_secs(30)))
            .add_handler(dummy_handler)
            .build();
            
        let job3 = JobBuilder::new("job3")
            .once(ScheduleTime::At(SystemTime::now() + Duration::from_secs(90)))
            .add_handler(dummy_handler)
            .build();
            
        scheduler.add_job(job1)?;
        scheduler.add_job(job2)?;
        scheduler.add_job(job3)?;
        
        let all_jobs = scheduler.list_all_jobs();
        assert_eq!(all_jobs.len(), 3);
        // Jobs sorted by next_run: time2 (30s), time1 (60s), time3 (90s)
        assert_eq!(all_jobs[0].name, Some("job2".to_string()));
        assert_eq!(all_jobs[1].name, Some("job1".to_string()));
        assert_eq!(all_jobs[2].name, Some("job3".to_string()));

        Ok(())
    }
    
    #[test]
    fn test_compute_next_run_recurring_intervals() {
        // Test different recurring interval calculations
        let now = SystemTime::now();
        
        // Test secondly
        let mut secondly_sched = Schedule {
            schedule_type: ScheduleType::Recurring(RecurringSchedule {
                interval: RecurringInterval::Secondly(5),
                next_run: now,
            }),
            max_runs: None,
            run_count: 0,
        };
        let next_secondly = Scheduler::compute_next_run(&mut secondly_sched).unwrap();
        assert_eq!(next_secondly, now + Duration::from_secs(5));
        
        // Test hourly
        let mut hourly_sched = Schedule {
            schedule_type: ScheduleType::Recurring(RecurringSchedule {
                interval: RecurringInterval::Hourly(2),
                next_run: now,
            }),
            max_runs: None,
            run_count: 0,
        };
        let next_hourly = Scheduler::compute_next_run(&mut hourly_sched).unwrap();
        assert_eq!(next_hourly, now + Duration::from_secs(2 * 3600));
        
        // Test daily
        let mut daily_sched = Schedule {
            schedule_type: ScheduleType::Recurring(RecurringSchedule {
                interval: RecurringInterval::Daily(1),
                next_run: now,
            }),
            max_runs: None,
            run_count: 0,
        };
        let next_daily = Scheduler::compute_next_run(&mut daily_sched).unwrap();
        assert_eq!(next_daily, now + Duration::from_secs(86400));
        
        // Test custom expression
        let mut custom_sched = Schedule {
            schedule_type: ScheduleType::Recurring(RecurringSchedule {
                interval: RecurringInterval::Custom { 
                    expression: "weekly".to_string(), 
                    frequency: 1 
                },
                next_run: now,
            }),
            max_runs: None,
            run_count: 0,
        };
        let next_custom = Scheduler::compute_next_run(&mut custom_sched).unwrap();
        assert_eq!(next_custom, now + Duration::from_secs(7 * 86400));
    }
    
    #[test]
    fn test_max_runs_limit() {
        let now = SystemTime::now();
        let mut sched = Schedule {
            schedule_type: ScheduleType::Recurring(RecurringSchedule {
                interval: RecurringInterval::Secondly(1),
                next_run: now,
            }),
            max_runs: Some(3),
            run_count: 3,  // Already reached max_runs
        };
        
        let next_run = Scheduler::compute_next_run(&mut sched);
        assert!(next_run.is_none());
    }
    
    #[test]
    fn test_peek_next_run() {
        let now = SystemTime::now();
        
        // Test recurring schedule
        let recurring_sched = Schedule {
            schedule_type: ScheduleType::Recurring(RecurringSchedule {
                interval: RecurringInterval::Secondly(1),
                next_run: now + Duration::from_secs(5),
            }),
            max_runs: None,
            run_count: 0,
        };
        
        let peeked = Scheduler::peek_next_run(&recurring_sched);
        assert_eq!(peeked.unwrap(), now + Duration::from_secs(5));
        
        // Test once schedule
        let once_sched = Schedule {
            schedule_type: ScheduleType::Once(now),
            max_runs: Some(1),
            run_count: 0,
        };
        
        let peeked_once = Scheduler::peek_next_run(&once_sched);
        assert!(peeked_once.is_none());
    }
    
    #[test]
    fn test_cron_schedule() -> Result<(), JobSchedulerError> {
        let mut scheduler = Scheduler::new();
        let cron_str = "0 0 * * * *"; // Run at midnight every day
        
        let job = JobBuilder::new("cron-job")
            .cron(cron_str)
            .add_handler(dummy_handler)
            .build();
            
        scheduler.add_job(job)?;
        
        assert!(scheduler.next_run().is_some());
        
        Ok(())
    }

    #[test]
    fn test_random_schedule() -> Result<(), JobSchedulerError> {
        let mut scheduler = Scheduler::new();
        
        // Create random scheduled job with fixed times for predictable testing
        let start = SystemTime::now() + Duration::from_secs(1);
        let end = SystemTime::now() + Duration::from_secs(5);
        
        let job = JobBuilder::new("random-job")
            .random(ScheduleTime::At(start), ScheduleTime::At(end))
            .add_handler(dummy_handler)
            .build();
            
        scheduler.add_job(job)?;
        
        assert_eq!(scheduler.jobs.len(), 1);
        assert!(scheduler.jobs[0].next_run.is_some());
        
        // The random time should be between start and end
        let next_run = scheduler.jobs[0].next_run.unwrap();
        assert!(next_run >= start && next_run <= end);
        
        Ok(())
    }
}
