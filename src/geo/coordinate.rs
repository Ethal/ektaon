// src/geo/coordinate.rs

use std::fmt;

use crate::error::CoordError;

// Indicates whether a coordinate is a latitude or a longitude.
// Used to apply correct bounds and valid directions.
#[derive(PartialEq)]
pub enum CoordinateKind {
    Latitude,
    Longitude,
}

// Identifies which field failed during parsing.
// This allows precise and explicit error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordField {
    Deg,
    Min,
    Sec,
    Dir,
}

// Human-readable representation of a coordinate field
// used in error messages.
impl fmt::Display for CoordField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CoordField::Deg => "degrees",
            CoordField::Min => "minutes",
            CoordField::Sec => "seconds",
            CoordField::Dir => "direction",
        };
        write!(f, "{s}")
    }
}

// Internal normalized representation of a parsed coordinate.
#[derive(PartialEq)]
pub struct Coordinate {
    pub deg: f64,
    pub min: f64,
    pub sec: f64,
    pub dir: Option<char>,
}

/* ---------------- LOW LEVEL VALIDATION ---------------- */

// Validates a parsed coordinate and converts it to decimal degrees.
// This function is the single source of truth for geographic rules.
pub fn coordinate_to_dd(coord: Coordinate, kind: CoordinateKind) -> Result<f64, CoordError> {
    let eps = 1e-12;

    if coord.dir.is_some() {
        // Validation degree / minutes / seconds
        if coord.deg < 0.0 {
            return Err(CoordError::InvalidDegree { deg: coord.deg });
        }
        if coord.min < 0.0 || coord.min >= 60.0 {
            return Err(CoordError::InvalidMinutes { min: coord.min });
        }
        if coord.sec < 0.0 || coord.sec >= 60.0 {
            return Err(CoordError::InvalidSeconds { sec: coord.sec });
        }
    }
    // Validation of geographical boundaries
    if kind == CoordinateKind::Latitude {
        if coord.deg > 90.0 + eps {
            return Err(CoordError::OutOfRange { deg: coord.deg });
        }
        if (coord.deg - 90.0).abs() < eps && (coord.min > 0.0 || coord.sec > 0.0) {
            return Err(CoordError::OutOfRange { deg: coord.deg });
        }
        if let Some(dir) = coord.dir {
            if !matches!(dir, 'N' | 'S') {
                return Err(CoordError::InvalidDirection(dir));
            }
        }
    }
    if kind == CoordinateKind::Longitude {
        if coord.deg > 180.0 + eps {
            return Err(CoordError::OutOfRange { deg: coord.deg });
        }
        if (coord.deg - 180.0).abs() < eps && (coord.min > 0.0 || coord.sec > 0.0) {
            return Err(CoordError::OutOfRange { deg: coord.deg });
        }
        if let Some(dir) = coord.dir {
            if !matches!(dir, 'E' | 'O' | 'W') {
                return Err(CoordError::InvalidDirection(dir));
            }
        }
    }

    // Conversion to decimal
    let mut value = coord.deg + (coord.min / 60.0) + (coord.sec / 3600.0);

    // Sign as per direction
    if let Some(dir) = coord.dir {
        if matches!(dir, 'S' | 'O' | 'W') {
            value = -value;
        }
    }
    Ok(value)
}
