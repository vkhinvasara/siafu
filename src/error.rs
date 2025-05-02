use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidSchedule(String),
    JobNotFound(String),
    ExecutionFailed(String),
    HandlerNotBuilt,
    MissingSchedule,
    TimeCalculationError,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidSchedule(msg) => write!(f, "Invalid schedule: {}", msg),
            Error::JobNotFound(id) => write!(f, "Job not found: {}", id),
            Error::ExecutionFailed(msg) => write!(f, "Job execution failed: {}", msg),
            Error::HandlerNotBuilt => write!(f, "Handler not built!"),
            Error::MissingSchedule => write!(f, "No schedule found!"),
            Error::TimeCalculationError => write!(f, "Error calculating target time"),
        }
    }
}