// src/error.rs

use thiserror::Error;

use crate::{
    geo::{DdmError, DmsError},
    util::HaversineError,
};

// Application-level errors.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Invalid header (missing or unreadable)")]
    InvalidHeader,

    #[error("Missing header field '{0}'")]
    MissingHeaderField(String),

    #[error("Invalid coordinate format on line {line} (expected: {expected})")]
    MixedCoordinateFormat { line: usize, expected: &'static str },

    #[error("Line {line}: invalid DMS ({source})")]
    InvalidDms { line: usize, source: DmsError },

    #[error("Line {line}: invalid DDM ({source})")]
    InvalidDdm { line: usize, source: DdmError },

    #[error("Distance calculation error: {0}")]
    Distance(#[from] HaversineError),
}
