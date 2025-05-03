use siafu::{JobBuilder, Scheduler};
use std::time::{SystemTime, Duration};
use std::thread;
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
        .repeat(3) // Run exactly 3 times
        .add_handler(move || {
            let mut count = job_counter.lock().unwrap();
            *count += 1;
            println!("Simple recurring job executed! (Execution #{})", *count);
        })
        .build();

    println!("Adding simple recurring job...");
    scheduler.add_job(simple_recurring_job)?;

    println!("Running scheduler...");

    // Run the scheduler until the job has executed 3 times or 15 seconds pass
    let start_time = SystemTime::now();
    loop {
        // Check elapsed time
        if SystemTime::now().duration_since(start_time)?.as_secs() >= 15 {
            println!("Scheduler timed out after 15 seconds.");
            break;
        }

        // Run pending jobs
        scheduler.run_pending()?;

        // Check execution count
        let current_count = *execution_counter.lock().unwrap();
        if current_count >= 3 {
            println!("Job executed 3 times. Exiting.");
            break;
        }

        // Print next scheduled run
        if let Some(next) = scheduler.next_run() {
            let duration = next.duration_since(SystemTime::now())
                .unwrap_or(Duration::from_secs(0));
            println!("Next job scheduled in {} seconds", duration.as_secs());
        } else {
             // Check count again in case the last run completed the job
             let final_count = *execution_counter.lock().unwrap();
             if final_count < 3 {
                 println!("Waiting for job to complete...");
             } else {
                println!("No more jobs scheduled and count reached.");
                break; // Exit if no more jobs and count met or exceeded
             }
        }

        // Sleep to avoid cpu spinning
        thread::sleep(Duration::from_millis(500)); // Check more frequently
    }

    let final_count = *execution_counter.lock().unwrap();
     if final_count < 3 {
        println!("Scheduler finished, but job only executed {} times.", final_count);
    }

    Ok(())
}
