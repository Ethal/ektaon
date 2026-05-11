// src/error.rs

use thiserror::Error;

use crate::geo::coordinate::CoordField;

// Application-level errors.
#[allow(clippy::enum_variant_names)]
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

// Errors specific to DDM parsing.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum DdmError {
    #[error("invalid DMS format")]
    InvalidFormat,
    #[error("invalid DDM field: {field}")]
    InvalidField { field: CoordField },
    #[error("invalid coord ({0})")]
    InvalidCoord(#[from] CoordError),
}

// Errors specific to DMS parsing.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum DmsError {
    #[error("invalid DMS format")]
    InvalidFormat,
    #[error("invalid DMS field: {field}")]
    InvalidField { field: CoordField },
    #[error("invalid coord ({0})")]
    InvalidCoord(#[from] CoordError),
}

// Errors related to numeric values and geographic limits.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum CoordError {
    #[error("coordinate out of range")]
    OutOfRange { deg: f64 },
    #[error("invalid degree value")]
    InvalidDegree { deg: f64 },
    #[error("invalid minutes value")]
    InvalidMinutes { min: f64 },
    #[error("invalid seconds value")]
    InvalidSeconds { sec: f64 },
    #[error("invalid direction `{0}`")]
    InvalidDirection(char),
}

// Errors specific to Haversine calculation.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum HaversineError {
    #[error("invalid distance")]
    InvalidDistance,

    // A negative distance should never happen.
    #[error("negative distance`{dist}`")]
    NegativeDistance { dist: f64 },
}
