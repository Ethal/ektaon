// src/geo/dd.rs

use crate::error::DdError;
use crate::geo::coordinate::{CoordField, Coordinate, CoordinateKind, coordinate_to_dd};

// Parses a DD string and converts it to decimal degrees.
pub fn dd_to_dd(input: &str, kind: CoordinateKind) -> Result<f64, DdError> {
    let deg: f64 = input
        .parse()
        .map_err(|_| DdError::InvalidField { field: CoordField::Deg })?;

    let min: f64 = 0.0;
    let sec: f64 = 0.0;

    if !deg.is_finite() || !min.is_finite() {
        return Err(DdError::InvalidFormat);
    }

    let coord = Coordinate {
        deg,
        min,
        sec,
        dir: None,
    };
    let value = coordinate_to_dd(coord, kind)?;

    Ok(value)
}
