use chrono::{DateTime, Utc};

pub enum ScheduleType {
    Once(DateTime<Utc>),
    Recurring(RecurringSchedule),
    Random(RandomSchedule),
}

pub struct Schedule {
    pub schedule_type: ScheduleType,
    pub max_runs: Option<u32>,
    pub run_count: u32,
}

pub struct RecurringSchedule {
    pub interval: RecurringInterval,
    pub next_run: DateTime<Utc>,
}

pub struct RandomSchedule {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

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