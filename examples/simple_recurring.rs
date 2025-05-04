use siafu::{JobBuilder, Scheduler};
use siafu::scheduler::types::RecurringInterval;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the scheduler
    let mut scheduler = Scheduler::new();
    let execution_counter = Arc::new(Mutex::new(0));

    // Example: Schedule a simple recurring job (3 times every 3 seconds)
    let job_counter = Arc::clone(&execution_counter);
    let simple_recurring_job = JobBuilder::new("simple-recurring-job")
        .recurring(RecurringInterval::Secondly(3), None) // Start immediately
        .max_repeat(3) // Run exactly 3 times
        .add_handler(move || {
            let mut count = job_counter.lock().unwrap();
            *count += 1;
            println!("Simple recurring job executed! (Execution #{})", *count);
        })
        .build();

    println!("Adding simple recurring job...");
    scheduler.add_job(simple_recurring_job)?;

    println!("Running scheduler...");

    // Block until all scheduled runs complete
    scheduler.run_non_blocking()?;
    // Print final execution count
    let final_count = *execution_counter.lock().unwrap();
    println!("Job executed {} times, exiting.", final_count);

    Ok(())
}
