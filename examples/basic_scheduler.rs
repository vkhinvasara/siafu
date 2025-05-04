use siafu::{JobBuilder, Scheduler};
use std::time::Duration;
use siafu::utils::time::ScheduleTime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the scheduler
    let mut scheduler = Scheduler::new();
    
    // Example 1: Schedule a job to run once 5 seconds from now
    let once_job = JobBuilder::new("once-job")
        .once(ScheduleTime::Delay(Duration::from_secs(5)))
        .add_handler(|| {
            println!("One-time job executed!");
        })
        .build();
        
    println!("Adding one-time job...");
    scheduler.add_job(once_job)?;
    
    // Example 2: Schedule a job using cron expression (runs every minute)
    let cron_job = JobBuilder::new("cron-job")
        .cron("0 * * * * * *")
        .add_handler(|| {
            println!("Cron job executed!");
        })
        .build();
        
    println!("Adding cron job...");
    scheduler.add_job(cron_job)?;
    
    // Example 3: Random scheduler (runs once at a random time between 5-15 seconds from now)
    let random_job = JobBuilder::new("random-job")
        .random(ScheduleTime::Delay(Duration::from_secs(5)), ScheduleTime::Delay(Duration::from_secs(15)))
        .add_handler(|| {
            println!("Random time job executed!");
        })
        .build();
        
    println!("Adding random time job...");
    scheduler.add_job(random_job)?;

    println!("Running scheduler...");

    // Block until no more jobs are scheduled
    scheduler.run_non_blocking()?;
    Ok(())
}