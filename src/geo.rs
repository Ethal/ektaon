// src/geo.rs

use regex::Regex;
use once_cell::sync::Lazy;

/* ---------------- DOMAIN TYPES ---------------- */

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
impl std::fmt::Display for CoordField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
// This structure is NOT exposed outside this module.
#[derive(PartialEq)]
struct Coordinate {
    deg: f64,
    min: f64,
    sec: f64,
    dir: char,
}


/* ---------------- LOW LEVEL VALIDATION ---------------- */

// Errors related to numeric values and geographic limits.
#[derive(Debug, thiserror::Error)]
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

// Validates a parsed coordinate and converts it to decimal degrees.
// This function is the single source of truth for geographic rules.
fn coordinate_to_dd(coord: Coordinate, kind: CoordinateKind) -> Result<f64, CoordError> {
    let eps = 1e-12;

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

    // Validation of geographical boundaries
    if kind == CoordinateKind::Latitude {
        if coord.deg > 90.0 + eps {
            return Err(CoordError::OutOfRange { deg: coord.deg });
        }
        if (coord.deg - 90.0).abs() < eps && (coord.min > 0.0 || coord.sec > 0.0) {
            return Err(CoordError::OutOfRange { deg: coord.deg });
        }
        if !matches!(coord.dir, 'N' | 'S') {
            return Err(CoordError::InvalidDirection(coord.dir));
        }
    }
    if kind == CoordinateKind::Longitude {
        if coord.deg > 180.0 + eps {
            return Err(CoordError::OutOfRange { deg: coord.deg });
        }
        if (coord.deg - 180.0).abs() < eps && (coord.min > 0.0 || coord.sec > 0.0) {
            return Err(CoordError::OutOfRange { deg: coord.deg });
        }
        if !matches!(coord.dir, 'E' | 'O' | 'W') {
            return Err(CoordError::InvalidDirection(coord.dir));
        }
    }

    // Conversion to decimal
    let mut value = coord.deg + (coord.min / 60.0) + (coord.sec / 3600.0);

    // Sign as per direction
    if matches!(coord.dir, 'S' | 'O' | 'W') {
        value = -value;
    }

    Ok(value)

}

/* ---------------- DMS ---------------- */

// Regex for Degrees / Minutes / Seconds format.
// Supports ASCII and Unicode symbols.
static DMS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?ix)^\s*
            (.+?)      # degrés (brut)
            \s*°\s*
            (.+?)      # minutes (brut)
            \s*['′]\s*
            (.+?)      # secondes (brut)
            \s*["″]\s*
            (.)        # direction (brut)
            \s*$"#
    ).expect("Invalid DMS regex")
});

// Errors specific to DMS parsing.
#[derive(Debug, thiserror::Error)]
pub enum DmsError {
    #[error("invalid DMS format")]
    InvalidFormat,
    #[error("invalid DMS field: {field}")]
    InvalidField { field: CoordField },
    #[error("invalid coord ({0})")]
    InvalidCoord(#[from] CoordError),
}

// Parses a DMS string and converts it to decimal degrees.
pub fn dms_to_dd(input: &str, kind: CoordinateKind) -> Result<f64, DmsError> {
    let caps = DMS_RE.captures(input)
        .ok_or(DmsError::InvalidFormat)?;

    let deg_str = caps.get(1).ok_or(DmsError::InvalidFormat)?.as_str().trim();
    let deg: f64 = deg_str.parse().map_err(|_| DmsError::InvalidField { field: CoordField::Deg })?;
    let min_str= caps.get(2).ok_or(DmsError::InvalidFormat)?.as_str().trim();
    let min: f64 = min_str.parse().map_err(|_| DmsError::InvalidField { field: CoordField::Min })?;
    let sec_str = caps.get(3).ok_or(DmsError::InvalidFormat)?.as_str().trim();
    let sec: f64 = sec_str.parse().map_err(|_| DmsError::InvalidField { field: CoordField::Sec })?;
    let dir_str = caps.get(4).ok_or(DmsError::InvalidFormat)?.as_str().trim();
    let dir = dir_str
        .chars()
        .next()
        .ok_or(DmsError::InvalidField { field: CoordField::Dir })?
        .to_ascii_uppercase();

    if !deg.is_finite() || !min.is_finite() || !sec.is_finite() {
        return Err(DmsError::InvalidFormat);
    }

    let coord = Coordinate { deg, min, sec, dir};
    let value = coordinate_to_dd(coord, kind)?;

    Ok(value)
}

/* ---------------- DDM ---------------- */

// Regex for Degrees / Decimal Minutes format.
// Supports ASCII and Unicode symbols.
static DDM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?ix)^\s*
            (.+?)      # degrés (brut)
            \s*°\s*
            (.+?)      # minutes (brut)
            \s*['′]\s*
            (.)        # direction (brut)
            \s*$"#
    ).expect("Invalid DMS regex")
});

// Errors specific to DDM parsing.
#[derive(Debug, thiserror::Error)]
pub enum DdmError {
    #[error("invalid DMS format")]
    InvalidFormat,
    #[error("invalid DDM field: {field}")]
    InvalidField { field: CoordField },
    #[error("invalid coord ({0})")]
    InvalidCoord(#[from] CoordError),
}

// Parses a DDM string and converts it to decimal degrees.
pub fn ddm_to_dd(input: &str, kind: CoordinateKind) -> Result<f64, DdmError> {
    let caps = DDM_RE.captures(input)
        .ok_or(DdmError::InvalidFormat)?;

    let deg_str = caps.get(1).ok_or(DdmError::InvalidFormat)?.as_str().trim();
    let deg: f64 = deg_str.parse().map_err(|_| DdmError::InvalidField { field:CoordField::Deg })?;
    let min_str= caps.get(2).ok_or(DdmError::InvalidFormat)?.as_str().trim();
    let min: f64 = min_str.parse().map_err(|_| DdmError::InvalidField { field:CoordField::Min })?;
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

    let coord = Coordinate { deg, min, sec, dir};
    let value = coordinate_to_dd(coord, kind)?;

    Ok(value)
}

/* ---------------- FORMATTING ---------------- */

// Converts decimal degrees to a DMS string.
// This function does not perform validation.
pub fn dd_to_dms(value: f64, kind: CoordinateKind) -> String {
    let dir = if kind == CoordinateKind::Latitude {
        if value >= 0.0 { 'N' } else { 'S' }
    } else {
        if value >= 0.0 { 'E' } else { 'W' }
    };

    let abs = value.abs();
    let deg = abs.floor();
    let min_f = (abs - deg) * 60.0;
    let min = min_f.floor();
    let sec = (min_f - min) * 60.0;

    format!("{}°{}'{:.2}\"{}", deg as i32, min as i32, sec, dir)
}
