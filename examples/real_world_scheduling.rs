use siafu::{JobBuilder, Scheduler};
use std::time::{SystemTime, Duration};
use std::thread;
// Removed direct cron imports; builder.cron takes a &str
// Add imports for ScheduleTime and RecurringInterval
use siafu::utils::time::ScheduleTime;
use siafu::scheduler::types::RecurringInterval;

// Simulate a database backup process
fn backup_database() -> anyhow::Result<()> {
    println!("📦 Starting database backup...");
    // Simulate some work
    thread::sleep(Duration::from_secs(1));
    println!("✅ Database backup completed successfully!");
    Ok(())
}

// Simulate sending a newsletter
fn send_newsletter() -> anyhow::Result<()> {
    println!("📧 Starting newsletter dispatch process...");
    // Simulate some work
    thread::sleep(Duration::from_secs(1));
    println!("✅ Newsletter sent to all subscribers!");
    Ok(())
}

// Simulate clearing cache
fn clear_cache() -> anyhow::Result<()> {
    println!("🗑️ Clearing application cache...");
    // Simulate some work
    thread::sleep(Duration::from_millis(500));
    println!("✅ Cache cleared successfully!");
    Ok(())
}

// Simulate health check
fn system_health_check() -> anyhow::Result<()> {
    println!("🔍 Running system health check...");
    // Simulate some work
    thread::sleep(Duration::from_millis(700));
    println!("✅ System health check completed: All services operational!");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the scheduler
    let mut scheduler = Scheduler::new();
    
    // 1. Schedule a daily database backup at midnight using cron
    let backup_job = JobBuilder::new("database-backup")
        .cron("0 0 0 * * * *")
        .add_handler(backup_database)
        .build();
    
    scheduler.add_job(backup_job)?;
    
    // 2. Schedule a weekly newsletter every Monday at 9 AM
    let newsletter_job = JobBuilder::new("weekly-newsletter")
        .cron("0 0 9 * * 1 *")
        .add_handler(send_newsletter)
        .build();
    
    scheduler.add_job(newsletter_job)?;
    
    // 3. Schedule cache clearing every 6 hours using recurring schedule
    let clear_cache_job = JobBuilder::new("cache-cleaner")
        .recurring(RecurringInterval::Hourly(6), Some(ScheduleTime::Delay(Duration::from_secs(10))))
        .add_handler(clear_cache)
        .build();
    
    scheduler.add_job(clear_cache_job)?;
    
    // 4. Schedule system health checks at random times between 1AM and 4AM
    // For this example, schedule between 15 and 25 seconds from now
    let health_check_job = JobBuilder::new("health-check")
        .random(ScheduleTime::Delay(Duration::from_secs(15)), ScheduleTime::Delay(Duration::from_secs(25)))
        .add_handler(system_health_check)
        .build();
    
    scheduler.add_job(health_check_job)?;
    
    println!("🚀 Job scheduler initialized with all maintenance jobs");
    println!("📅 Running scheduler for demo (30 seconds, jobs scheduled closer for demonstration)");
    
    // For this example, run for 30 seconds
    let start_time = SystemTime::now();
    while SystemTime::now().duration_since(start_time)?.as_secs() < 30 {
        scheduler.run_pending()?;
        
        // Display next job info
        if let Some(next) = scheduler.next_run() {
            let duration = next.duration_since(SystemTime::now())
                .unwrap_or(Duration::from_secs(0));
            println!("⏰ Next job scheduled in {} seconds", duration.as_secs());
            
            // List all jobs
            println!("\n📋 Current job schedule:");
            for (i, job) in scheduler.list_all_jobs().iter().enumerate() {
                // Name is an Option<String> field on JobBuilder
                println!("  {}. Job: {}", i + 1, job.name.as_ref().unwrap_or(&"Unnamed job".to_string()));
            }
            println!();
        }
        
        // Sleep to avoid CPU spinning
        thread::sleep(Duration::from_secs(1));
    }
    
    println!("✨ Demo completed!");
    Ok(())
}