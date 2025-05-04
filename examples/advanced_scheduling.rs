use siafu::{JobBuilder, Scheduler};
use std::time::Duration;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

// Import rand for the random boolean generation
use rand::Rng;
// Add imports for ScheduleTime and RecurringInterval
use siafu::utils::time::ScheduleTime;
use siafu::scheduler::types::RecurringInterval;

// Shared state to simulate job dependencies and error tracking
struct AppState {
    job_results: HashMap<String, bool>,
    error_count: HashMap<String, u32>,
}

// Non-capturing function handlers for each job type
fn extract_job_handler() {
    println!("📥 Starting data extraction process...");
    
    // Simulate work with a 50% chance of success
    let success = rand::rng().random_bool(0.5);
    
    if success {
        println!("✅ Data extraction completed successfully!");
        // We'll update state separately
    } else {
        println!("❌ Error: Data extraction failed!");
        // We'll update state separately
        // If error handling is needed, the handler should panic or use shared state.
        // For simplicity, we just print here.
    }
}

fn transform_job_handler() {
    println!("🔄 Starting data transformation...");
    println!("✅ Data transformation completed");
}

fn load_job_handler() {
    println!("📤 Starting data loading process...");
    println!("✅ Data loaded successfully into target systems");
}

fn monitor_job_handler() {
    println!("\n📊 Job status monitor running...");
    // We'll handle the actual monitoring separately
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize shared state for job tracking
    let state = Arc::new(Mutex::new(AppState {
        job_results: HashMap::new(),
        error_count: HashMap::new(),
    }));
    
    // Initialize the scheduler
    let mut scheduler = Scheduler::new();
    
    // Step 1: Data extraction job
    let extract_job = JobBuilder::new("data-extract")
        .once(ScheduleTime::Delay(Duration::from_secs(3)))
        .add_handler(extract_job_handler)
        .build();
    
    let extract_state = Arc::clone(&state);
    let extract_job_id = extract_job.name.clone();
    scheduler.add_job(extract_job)?;
    
    // Step 2: Transform job (depends on extract)
    let transform_job = JobBuilder::new("transform-data")
        .once(ScheduleTime::Delay(Duration::from_secs(8)))
        .add_handler(transform_job_handler)
        .build();
    
    let transform_state = Arc::clone(&state);
    let transform_job_id = transform_job.name.clone();
    scheduler.add_job(transform_job)?;
    
    // Step 3: Load job (depends on transform)
    let load_job = JobBuilder::new("load-data")
        .once(ScheduleTime::Delay(Duration::from_secs(13)))
        .add_handler(load_job_handler)
        .build();
    
    let load_state = Arc::clone(&state);
    let load_job_id = load_job.name.clone();
    scheduler.add_job(load_job)?;
    
    // Monitoring job that runs every 5 seconds
    let monitor_job = JobBuilder::new("job-monitor")
        .recurring(RecurringInterval::Secondly(5), Some(ScheduleTime::Delay(Duration::from_secs(5))))
        .add_handler(monitor_job_handler)
        .build();
    
    let monitor_state = Arc::clone(&state);
    scheduler.add_job(monitor_job)?;
    
    println!("🚀 Advanced job orchestration system started");
    println!("📋 Jobs scheduled with dependencies: extract → transform → load");
    println!("🔍 Monitor will check job status every 5 seconds\n");
    
    // Block until all scheduled jobs have run
    scheduler.run_non_blocking()?;

    println!("✨ Advanced scheduler demo completed!");

    // Final status report
    println!("\n📑 Final Job Status:");
    let state = state.lock().unwrap();
    for (job_name, success) in &state.job_results {
        let status = if *success { "✅ Succeeded" } else { "❌ Failed" };
        println!("  - {}: {}", job_name, status);
    }

    Ok(())
}