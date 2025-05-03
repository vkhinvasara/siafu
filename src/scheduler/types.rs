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