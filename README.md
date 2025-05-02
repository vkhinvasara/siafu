## Ergonomic Job Scheduler Library

<p align="center">
  <img src="siafu.png" alt="Siafu (Safari Ants)" width="300">
</p>

A flexible and ergonomic job scheduling library for Rust applications with an intuitive, fluent public API.

### Why "Siafu"?

The name "Siafu" refers to safari ants (also known as driver ants or army ants), which are famous for their highly organized and efficient colony operations. These fascinating insects:

- Work in perfect coordination with specialized roles for different tasks
- Execute complex operations through distributed intelligence
- Can adapt their scheduling and routing based on environmental changes
- Form living bridges and structures to overcome obstacles

Just like these remarkable ants, the Siafu library excels at organizing, scheduling, and executing tasks in a coordinated manner. The library embodies the efficiency, reliability, and adaptability of safari ants' work patterns, making it the perfect metaphor for a job scheduling system.

### Features

- Schedule tasks to run on:
  - Specific dates/times, e.g., 20 Sept 10:00 pm
  - Recurring intervals, e.g., hourly, daily, weekly, monthly
  - Random intervals, e.g., between 9-10 am
  - Cron expressions for complex scheduling patterns
- Set limits on recurring jobs: hourly 5 times or daily x times, first Friday of every month
- Error handling and job monitoring capabilities
- Fluent builder API for easy job configuration

### Usage Examples

#### Basic Usage

```rust
use job_scheduler::{JobBuilder, Scheduler};
use std::time::{SystemTime, Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the scheduler
    let mut scheduler = Scheduler::new();
    
    // Schedule a job to run once 5 seconds from now
    let job = JobBuilder::new("one-time-job", "A job that runs once")
        .once(SystemTime::now() + Duration::from_secs(5))
        .add_handler(|| {
            println!("One-time job executed!");
            Ok(())
        })
        .build();
        
    scheduler.add_job(job)?;
    
    // Run the scheduler checking for pending jobs
    loop {
        scheduler.run_pending()?;
        std::thread::sleep(Duration::from_secs(1));
    }
}
```

### Scheduling Types

#### One-time jobs

```rust
// Run once at a specific time
let job = JobBuilder::new("once-job", "")
    .once(SystemTime::now() + Duration::from_secs(60))
    .add_handler(|| { println!("Executed!"); Ok(()) })
    .build();
```

#### Recurring jobs

```rust
// Run every 10 seconds
let job = JobBuilder::new("recurring-job", "")
    .recurring(job_scheduler::scheduler::types::RecurringSchedule {
        interval: job_scheduler::scheduler::types::RecurringInterval::Secondly(Some(10)),
        next_run: SystemTime::now() + Duration::from_secs(10),
    })
    .repeat(5) // Run up to 5 times
    .add_handler(|| { println!("Recurring job executed!"); Ok(()) })
    .build();
```

#### Cron jobs

```rust
use std::str::FromStr;
use cron::Schedule;

// Run at specific times using cron expression
let cron_schedule = Schedule::from_str("0 0 9 * * 1-5 *")?; // 9 AM on weekdays
let job = JobBuilder::new("cron-job", "")
    .cron(cron_schedule)
    .add_handler(|| { println!("Cron job executed!"); Ok(()) })
    .build();
```

#### Random time jobs

```rust
// Run once at a random time within a range
let start = SystemTime::now() + Duration::from_secs(60);
let end = SystemTime::now() + Duration::from_secs(300);
let job = JobBuilder::new("random-job", "")
    .random(start, end)
    .add_handler(|| { println!("Random time job executed!"); Ok(()) })
    .build();
```

### Examples

Check out the examples directory for more comprehensive examples:

- **basic_scheduler.rs**: Demonstrates all scheduling types with simple examples
- **real_world_scheduling.rs**: Shows practical applications like database backups, newsletter sending, etc.
- **advanced_scheduling.rs**: Demonstrates job dependencies, error handling, and job monitoring

To run an example:

```bash
cargo run --example basic_scheduler
```