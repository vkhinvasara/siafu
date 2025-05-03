use siafu::{JobBuilder, Scheduler};
use std::time::{SystemTime, Duration};
use std::str::FromStr;
use cron::Schedule;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the scheduler
    let mut scheduler = Scheduler::new();
    
    // Example 1: Schedule a job to run once 5 seconds from now
    let once_job = JobBuilder::new("once-job", "A job that runs once after 5 seconds")
        .once(SystemTime::now() + Duration::from_secs(5))
        .add_handler(|| {
            println!("One-time job executed!");
            Ok(())
        })
        .build();
        
    println!("Adding one-time job...");
    scheduler.add_job(once_job)?;
    
    // Example 2: Schedule a recurring job that runs every 3 seconds
    let recurring_job = JobBuilder::new("recurring-job", "A job that runs every 3 seconds")
        .recurring(siafu::scheduler::types::RecurringSchedule {
            interval: siafu::scheduler::types::RecurringInterval::Secondly(Some(3)),
            next_run: SystemTime::now() + Duration::from_secs(3),
        })
        .repeat(5) // Run up to 5 times
        .add_handler(|| {
            println!("Recurring job executed!");
            Ok(())
        })
        .build();
        
    println!("Adding recurring job...");
    scheduler.add_job(recurring_job)?;
    
    // Example 3: Schedule a job using cron expression (runs every minute)
    let cron_schedule = Schedule::from_str("0 * * * * * *")?;
    let cron_job = JobBuilder::new("cron-job", "A job that runs using cron schedule")
        .cron(cron_schedule)
        .add_handler(|| {
            println!("Cron job executed!");
            Ok(())
        })
        .build();
        
    println!("Adding cron job...");
    scheduler.add_job(cron_job)?;
    
    // Example 4: Random scheduler (runs once at a random time between 5-15 seconds from now)
    let start = SystemTime::now() + Duration::from_secs(5);
    let end = SystemTime::now() + Duration::from_secs(15);
    let random_job = JobBuilder::new("random-job", "A job that runs once at a random time")
        .random(start, end)
        .add_handler(|| {
            println!("Random time job executed!");
            Ok(())
        })
        .build();
        
    println!("Adding random time job...");
    scheduler.add_job(random_job)?;
    
    println!("Running scheduler...");
    
    // Run the scheduler for 30 seconds checking for pending jobs
    let start_time = SystemTime::now();
    while SystemTime::now().duration_since(start_time)?.as_secs() < 30 {
        scheduler.run_pending()?;
        
        // Print next scheduled run
        if let Some(next) = scheduler.next_run() {
            let duration = next.duration_since(SystemTime::now())
                .unwrap_or(Duration::from_secs(0));
            println!("Next job scheduled in {} seconds", duration.as_secs());
        } else {
            println!("No more jobs scheduled");
        }
        
        // Sleep to avoid cpu spinning
        thread::sleep(Duration::from_secs(1));
    }
    
    Ok(())
}