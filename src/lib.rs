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

pub mod job;
pub mod scheduler;
pub mod error;

pub use job::JobBuilder;
pub use scheduler::*;

/// Current version of the Siafu library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");