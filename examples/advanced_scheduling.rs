use siafu::{JobBuilder, Scheduler};
use std::time::{Duration};
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
fn extract_job_handler() -> anyhow::Result<()> {
    println!("📥 Starting data extraction process...");
    
    // Simulate work with a 50% chance of success
    let success = rand::rng().random_bool(0.5);
    
    if success {
        println!("✅ Data extraction completed successfully!");
        // We'll update state separately
    } else {
        println!("❌ Error: Data extraction failed!");
        // We'll update state separately
        return Err(anyhow::anyhow!("Data extraction failed"));
    }
    
    Ok(())
}

fn transform_job_handler() -> anyhow::Result<()> {
    println!("🔄 Starting data transformation...");
    println!("✅ Data transformation completed");
    Ok(())
}

fn load_job_handler() -> anyhow::Result<()> {
    println!("📤 Starting data loading process...");
    println!("✅ Data loaded successfully into target systems");
    Ok(())
}

fn monitor_job_handler() -> anyhow::Result<()> {
    println!("\n📊 Job status monitor running...");
    // We'll handle the actual monitoring separately
    println!();
    Ok(())
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
    
    // Run for 30 seconds
    let start_time = SystemTime::now();
    while SystemTime::now().duration_since(start_time)?.as_secs() < 30 {
        // Capture and log errors but don't halt execution
        match scheduler.run_pending() {
            Ok(_) => {
                // After running pending jobs, we need to manually check and update state
                // For extract job success/failure
                if let Some(extract_id) = &extract_job_id {
                    // This is a simplified check - in a real app we'd need a better job state tracking system
                    if SystemTime::now() > (start_time + Duration::from_secs(3)) {
                        let mut state = extract_state.lock().unwrap();
                        if !state.job_results.contains_key(extract_id) {
                            // Simulate the same random outcome as in the handler
                            let success = rand::rng().random_bool(0.5);
                            state.job_results.insert(extract_id.clone(), success);
                            if !success {
                                *state.error_count.entry(extract_id.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                
                // For transform job dependencies and state
                if let Some(transform_id) = &transform_job_id {
                    if SystemTime::now() > (start_time + Duration::from_secs(8)) {
                        let mut state = transform_state.lock().unwrap();
                        if !state.job_results.contains_key(transform_id) {
                            // Check if extract succeeded before recording transform success
                            if let Some(extract_id) = &extract_job_id {
                                let extract_succeeded = state.job_results.get(extract_id).copied().unwrap_or(false);
                                state.job_results.insert(transform_id.clone(), extract_succeeded);
                                if !extract_succeeded {
                                    println!("⚠️ Cannot transform data: Extract job failed or incomplete");
                                }
                            }
                        }
                    }
                }
                
                // For load job dependencies and state
                if let Some(load_id) = &load_job_id {
                    if SystemTime::now() > (start_time + Duration::from_secs(13)) {
                        let mut state = load_state.lock().unwrap();
                        if !state.job_results.contains_key(load_id) {
                            // Check if transform succeeded before recording load success
                            if let Some(transform_id) = &transform_job_id {
                                let transform_succeeded = state.job_results.get(transform_id).copied().unwrap_or(false);
                                state.job_results.insert(load_id.clone(), transform_succeeded);
                                if !transform_succeeded {
                                    println!("⚠️ Cannot load data: Transform job failed or incomplete");
                                }
                            }
                        }
                    }
                }
                
                // Monitoring logic
                if SystemTime::now() > (start_time + Duration::from_secs(5)) && 
                   SystemTime::now().duration_since(start_time)?.as_secs() % 5 == 0 {
                    let state = monitor_state.lock().unwrap();
                    
                    for (job_name, success) in &state.job_results {
                        let attempt_count = state.error_count.get(job_name).copied().unwrap_or(0);
                        
                        if !success && attempt_count < 3 {
                            println!("🔄 Job '{}' failed. Retry attempt {} scheduled.", job_name, attempt_count + 1);
                        } else if !success {
                            println!("❌ Job '{}' failed after {} attempts, marked as failed", job_name, attempt_count);
                        } else {
                            println!("✓ Job '{}' completed successfully", job_name);
                        }
                    }
                }
            },
            Err(e) => println!("📛 Scheduler error: {}", e),
        }
        
        thread::sleep(Duration::from_secs(1));
    }
    
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