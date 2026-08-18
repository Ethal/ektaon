// src/geo/dms.rs

use regex::Regex;
use std::sync::LazyLock;

use crate::error::DmsError;
use crate::geo::coordinate::{CoordField, Coordinate, CoordinateKind, coordinate_to_dd};

/* ---------------- DMS ---------------- */

// Regex for Degrees / Minutes / Seconds format.
// Supports ASCII and Unicode symbols.
static DMS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?ix)^\s*
            (.+?)      # degrés (brut)
            \s*°\s*
            (.+?)      # minutes (brut)
            \s*['′]\s*
            (.+?)      # secondes (brut)
            \s*["″]\s*
            (.)        # direction (brut)
            \s*$"#,
    )
    .expect("Invalid DMS regex")
});

// Parses a DMS string and converts it to decimal degrees.
pub fn dms_to_dd(input: &str, kind: CoordinateKind) -> Result<f64, DmsError> {
    let caps = DMS_RE.captures(input).ok_or(DmsError::InvalidFormat)?;

    let deg_str = caps.get(1).ok_or(DmsError::InvalidFormat)?.as_str().trim();
    let deg: f64 = deg_str
        .parse()
        .map_err(|_| DmsError::InvalidField { field: CoordField::Deg })?;
    let min_str = caps.get(2).ok_or(DmsError::InvalidFormat)?.as_str().trim();
    let min: f64 = min_str
        .parse()
        .map_err(|_| DmsError::InvalidField { field: CoordField::Min })?;
    let sec_str = caps.get(3).ok_or(DmsError::InvalidFormat)?.as_str().trim();
    let sec: f64 = sec_str
        .parse()
        .map_err(|_| DmsError::InvalidField { field: CoordField::Sec })?;
    let dir_str = caps.get(4).ok_or(DmsError::InvalidFormat)?.as_str().trim();
    let dir = dir_str
        .chars()
        .next()
        .ok_or(DmsError::InvalidField { field: CoordField::Dir })?
        .to_ascii_uppercase();

    if !deg.is_finite() || !min.is_finite() || !sec.is_finite() {
        return Err(DmsError::InvalidFormat);
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
