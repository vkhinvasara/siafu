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

#[derive(Clone)]
pub enum RecurringInterval {
    Secondly(Option<u8>), 
    Hourly(Option<u8>),   
    Daily(Option<u8>),    
    Weekly(Option<u8>),   
    Monthly(Option<u8>),  
    Custom { 
        expression: String, 
        frequency: Option<u8>, 
    },
}