//! # Siafu
//! 
//! An ergonomic job scheduling library for Rust.
//! 
//! ## Features
//! 
//! - Schedule tasks to run on specific dates/times
//! - Set up recurring intervals (hourly, daily, weekly, monthly)
//! - Schedule jobs using cron expressions for complex patterns
//! - Run jobs at random times within a specified range
//! - Set limits on recurring jobs
//! - Error handling and job monitoring capabilities
//! - Fluent builder API for easy job configuration
//! 
//! # Examples
//!
//! Basic usage:
//!
//! ```rust
//! use siafu::{JobBuilder, ScheduleTime, SchedulerError};
//! use std::time::{Duration, SystemTime};
//!
//! fn main() -> Result<(), SchedulerError> {
//!     // One-off job after 5 seconds
//!     let mut job = JobBuilder::new("example_once")
//!         .once(ScheduleTime::Delay(Duration::from_secs(5)))
//!         .add_handler(|| println!("Hello after delay!"))
//!         .build();
//!
//!     // Recurring job every 10 seconds
//!     let _recurring = JobBuilder::new("example_recurring")
//!         .every(Duration::from_secs(10), None)
//!         .add_handler(|| println!("Recurring task"))
//!         .build();
//!
//!     // Cron-based job
//!     let _cron = JobBuilder::new("example_cron")
//!         .cron("0 0 * * * * *")
//!         .add_handler(|| println!("Hourly task on the hour"))
//!         .build();
//!
//!     // Run the one-off job
//!     job.run()?;
//!     Ok(())
//! }
//! ```

pub mod job;
pub mod scheduler;
pub mod error;
pub mod utils;

pub use job::JobBuilder;
pub use scheduler::*;
pub use utils::time::{ScheduleTime, ScheduleTimeError};
pub use error::Error as SchedulerError;
/// Current version of the Siafu library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");