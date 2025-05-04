use siafu::{JobBuilder, Scheduler};
use std::thread;
use std::time::Duration;
use siafu::utils::time::ScheduleTime;
use siafu::scheduler::types::RecurringInterval;

// Simulate a database backup process
fn backup_database() {
    println!("📦 Starting database backup...");
    // Simulate some work
    thread::sleep(Duration::from_secs(1));
    println!("✅ Database backup completed successfully!");
    
}

// Simulate sending a newsletter
fn send_newsletter() {
    println!("📧 Starting newsletter dispatch process...");
    // Simulate some work
    thread::sleep(Duration::from_secs(1));
    println!("✅ Newsletter sent to all subscribers!");
    
}

// Simulate clearing cache
fn clear_cache() {
    println!("🗑️ Clearing application cache...");
    // Simulate some work
    thread::sleep(Duration::from_millis(500));
    println!("✅ Cache cleared successfully!");
    
}

// Simulate health check
fn system_health_check()  {
    println!("🔍 Running system health check...");
    // Simulate some work
    thread::sleep(Duration::from_millis(700));
    println!("✅ System health check completed: All services operational!");
    
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
    
    // Block until all scheduled jobs have run
    scheduler.run_non_blocking()?;
    println!("✨ Demo completed!");
    Ok(())
}