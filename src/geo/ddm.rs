// src/geo/ddm.rs

use regex::Regex;
use std::sync::LazyLock;

use crate::error::DdmError;
use crate::geo::coordinate::{CoordField, Coordinate, CoordinateKind, coordinate_to_dd};

/* ---------------- DDM ---------------- */

// Regex for Degrees / Decimal Minutes format.
// Supports ASCII and Unicode symbols.
static DDM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?ix)^\s*
            (.+?)      # degrés (brut)
            \s*°\s*
            (.+?)      # minutes (brut)
            \s*['′]\s*
            (.)        # direction (brut)
            \s*$"#,
    )
    .expect("Invalid DMS regex")
});

// Parses a DDM string and converts it to decimal degrees.
pub fn ddm_to_dd(input: &str, kind: CoordinateKind) -> Result<f64, DdmError> {
    let caps = DDM_RE.captures(input).ok_or(DdmError::InvalidFormat)?;

    let deg_str = caps.get(1).ok_or(DdmError::InvalidFormat)?.as_str().trim();
    let deg: f64 = deg_str
        .parse()
        .map_err(|_| DdmError::InvalidField { field: CoordField::Deg })?;
    let min_str = caps.get(2).ok_or(DdmError::InvalidFormat)?.as_str().trim();
    let min: f64 = min_str
        .parse()
        .map_err(|_| DdmError::InvalidField { field: CoordField::Min })?;
    let dir_str = caps.get(3).ok_or(DdmError::InvalidFormat)?.as_str().trim();
    let dir = dir_str
        .chars()
        .next()
        .ok_or(DdmError::InvalidField { field: CoordField::Dir })?
        .to_ascii_uppercase();

    let sec: f64 = 0.0;

    if !deg.is_finite() || !min.is_finite() {
        return Err(DdmError::InvalidFormat);
    }

    let coord = Coordinate {
        deg,
        min,
        sec,
        dir: Some(dir),
    };
    let value = coordinate_to_dd(coord, kind)?;

    Ok(value)
}
