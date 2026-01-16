// src/main.rs

/*
ARCHITECTURE OVERVIEW

This binary is a CSV-to-CSV geographic distance processor.

High-level flow:
1. Parse CLI arguments (input/output paths, coordinate format, strict mode).
2. Validate CSV headers.
3. Read input rows and parse coordinates according to the selected format:
   - DD  → direct numeric values
   - DMS → parsed and validated strings
   - DDM → parsed and validated strings
4. Normalize all coordinates to:
   - decimal degrees (DD)
   - formatted DMS strings
5. Compute:
   - Haversine distance (km / miles)
   - near-equality flags (lat / lon / both)
6. Write enriched rows to the output CSV.

Key design choices:
- Coordinate format is global (no mixed formats per file).
- All computations use normalized decimal degrees.
- Errors are handled per-line in permissive mode, or fail-fast in strict mode.
- Parsing, geometry, and math logic are isolated in `geo` and `util` modules.

The main module focuses on orchestration and I/O only.
*/

use std::fs::File;
use std::path::PathBuf;
use std::collections::HashSet;

use clap::Parser;
use clap::ValueEnum;
use csv::{ReaderBuilder, Writer};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod util;
use crate::util::KM_TO_MILES;
use crate::util::HaversineError;
use crate::util::GeoTolerance;
use crate::util::Nearly;
use crate::util::round;
use crate::util::haversine;
use crate::util::compute_nearly;

mod geo;
use crate::geo::CoordinateKind;
use crate::geo::dd_to_dms;
use crate::geo::dms_to_dd;
use crate::geo::ddm_to_dd;
use crate::geo::DmsError;
use crate::geo::DdmError;

/* ---------------- CONSTANTES ---------------- */

// Required CSV headers (order-independent).
const REQUIRED_HEADERS: &[&str] = &[
    "name_a",
    "lat_a",
    "lon_a",
    "name_b",
    "lat_b",
    "lon_b",
];

/* ---------------- CLI ---------------- */

// Command-line interface definition.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Input CSV file path
    #[arg(short, long)]
    input: PathBuf,

    /// Output CSV file path
    #[arg(short, long)]
    output: PathBuf,

    /// Coordinate input format
    #[arg(short ='f', long, value_enum)]
    input_format: InputFormat,

    /// Strict mode: stop on first error
    #[arg(long)]
    strict: bool,
}

// Supported coordinate formats.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum InputFormat {
    Dd,
    Dms,
    Ddm,
}

/* ---------------- MAIN ERROR ---------------- */

// Application-level errors.
#[derive(Error, Debug)]
enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Invalid header (missing or unreadable)")]
    InvalidHeader,

    #[error("Missing header field '{0}'")]
    MissingHeaderField(String),

    #[error("Invalid coordinate format on line {line} (expected: {expected})")]
    MixedCoordinateFormat {
        line: usize,
        expected: &'static str,
    },

    #[error("Line {line}: invalid DMS ({source})")]
    InvalidDms {
        line: usize,
        source: DmsError,
    },

    #[error("Line {line}: invalid DDM ({source})")]
    InvalidDdm {
        line: usize,
        source: DdmError,
    },

    #[error("Distance calculation error: {0}")]
    Distance(#[from] HaversineError),
}

/* ---------------- INPUT CSV STRUCTS ---------------- */

// Decimal degrees input.
#[derive(Debug, Deserialize)]
struct InputDecimal {
    name_a: String,
    lat_a: f64,
    lon_a: f64,
    name_b: String,
    lat_b: f64,
    lon_b: f64,
}

// String-based input (DMS / DDM).
#[derive(Debug, Deserialize)]
struct InputString {
    name_a: String,
    lat_a: String,
    lon_a: String,
    name_b: String,
    lat_b: String,
    lon_b: String,
}

/* ---------------- OUTPUT CSV STRUCTS ---------------- */

// Output CSV record (fully normalized).
#[derive(Debug, Serialize)]
struct OutputRecord {
    id: u64,

    name_a: String,
    lat_a_in: String,
    lon_a_in: String,
    lat_a_dd: f64,
    lon_a_dd: f64,
    lat_a_dms: String,
    lon_a_dms: String,

    name_b: String,
    lat_b_in: String,
    lon_b_in: String,
    lat_b_dd: f64,
    lon_b_dd: f64,
    lat_b_dms: String,
    lon_b_dms: String,

    distance_km: f64,
    distance_miles: f64,
    nearly_lat: bool,
    nearly_lon: bool,
    nearly_both: bool,
}

/* ---------------- NORMALIZED ---------------- */

// Normalized coordinate representation.
#[derive(Debug, Clone)]
struct NormalizedCoord {
    input: String,  // original input string
    dd: f64,        // decimal degrees
    dms: String,    // formatted DMS output
}

// Normalized geographic point.
#[derive(Debug, Clone)]
struct NormalizedPoint {
    name: String,
    lat: NormalizedCoord,
    lon: NormalizedCoord,
}

// Normalized geographic point.
#[derive(Debug, Clone)]
struct NormalizedGeo {
    a: NormalizedPoint,
    b: NormalizedPoint,
}

// Distance and comparison metrics.
#[derive(Debug)]
struct DistanceMetrics {
    km: f64,
    miles: f64,
    nearly: Nearly,
}

/* ---------------- MAIN ---------------- */

fn main() -> Result<(), AppError> {

    // Parse CLI arguments.
    let cli = Cli::parse();

    // CSV reader / writer setup.
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(cli.input)?;
    let mut writer = Writer::from_writer(File::create(cli.output)?);

    // Validate required headers.
    let headers = reader.headers()
        .map_err(|_| AppError::InvalidHeader)?;

    let headers: HashSet<_> = headers.iter().collect();
    for &h in REQUIRED_HEADERS {
        if !headers.contains(h) {
            return Err(AppError::MissingHeaderField(h.to_string()));
        }
    }

    // Processing counters.
    let mut id: u64 = 1;
    let mut invalid: u64 = 0;
    let mut line_no = 1;

    // Dispatch based on input format.
    match cli.input_format {
        InputFormat::Dms => {
            for row in reader.deserialize::<InputString>() {
                line_no += 1;
                let r = match row {
                    Ok(v) => v,
                    Err(_) => {
                        invalid += 1;
                        if cli.strict {
                            return Err(AppError::MixedCoordinateFormat {
                                line: line_no,
                                expected: "DMS",
                            });
                        }
                        continue;
                    }
                };

                // Parse DMS coordinates.
                let (lat_a_dd, lon_a_dd, lat_b_dd, lon_b_dd) = match (
                    dms_to_dd(&r.lat_a, CoordinateKind::Latitude),
                    dms_to_dd(&r.lon_a, CoordinateKind::Longitude),
                    dms_to_dd(&r.lat_b, CoordinateKind::Latitude),
                    dms_to_dd(&r.lon_b, CoordinateKind::Longitude),
                ) {
                    (Ok(a), Ok(b), Ok(c), Ok(d)) => (a, b, c, d),
                    (Err(e), _, _, _)
                    | (_, Err(e), _, _)
                    | (_, _, Err(e), _)
                    | (_, _, _, Err(e)) => {
                        invalid += 1;
                        if cli.strict {
                            return Err(AppError::InvalidDms {
                                line: line_no,
                                source: e,
                            });
                        }
                        continue;
                    }
                };

                let geo = build_normalized_geo(
                    r.name_a,
                    r.lat_a,
                    r.lon_a,
                    lat_a_dd,
                    lon_a_dd,
                    r.name_b,
                    r.lat_b,
                    r.lon_b,
                    lat_b_dd,
                    lon_b_dd,
                );

                process_geo(&mut writer, &geo, &mut id, cli.strict, &mut invalid)?;
            }
        }
        InputFormat::Ddm => {
            for row in reader.deserialize::<InputString>() {
                line_no += 1;
                let r = match row {
                    Ok(v) => v,
                    Err(_) => {
                        invalid += 1;
                        if cli.strict {
                            return Err(AppError::MixedCoordinateFormat {
                                line: line_no,
                                expected: "DDM",
                            });
                        }
                        continue;
                    }
                };

                // Parse DDM coordinates.
                let (lat_a_dd, lon_a_dd, lat_b_dd, lon_b_dd) = match (
                    ddm_to_dd(&r.lat_a, CoordinateKind::Latitude),
                    ddm_to_dd(&r.lon_a, CoordinateKind::Longitude),
                    ddm_to_dd(&r.lat_b, CoordinateKind::Latitude),
                    ddm_to_dd(&r.lon_b, CoordinateKind::Longitude),
                ) {
                    (Ok(a), Ok(b), Ok(c), Ok(d)) => (a, b, c, d),
                    (Err(e), _, _, _)
                    | (_, Err(e), _, _)
                    | (_, _, Err(e), _)
                    | (_, _, _, Err(e)) => {
                        invalid += 1;
                        if cli.strict {
                            return Err(AppError::InvalidDdm {
                                line: line_no,
                                source: e,
                            });                    }
                        continue;
                    }
                };

                let geo = build_normalized_geo(
                    r.name_a,
                    r.lat_a,
                    r.lon_a,
                    lat_a_dd,
                    lon_a_dd,
                    r.name_b,
                    r.lat_b,
                    r.lon_b,
                    lat_b_dd,
                    lon_b_dd,
                );

                process_geo(&mut writer, &geo, &mut id, cli.strict, &mut invalid)?;
            }
        }
        InputFormat::Dd => {
            for row in reader.deserialize::<InputDecimal>() {
                line_no += 1;
                let r = match row {
                    Ok(v) => v,
                    Err(_) => {
                        invalid += 1;
                        if cli.strict {
                            return Err(AppError::MixedCoordinateFormat {
                                line: line_no,
                                expected: "dd",
                            });
                        }
                        continue;
                    }
                };

                let geo = build_normalized_geo(
                    r.name_a,
                    r.lat_a.to_string(),
                    r.lon_a.to_string(),
                    r.lat_a,
                    r.lon_a,
                    r.name_b,
                    r.lat_b.to_string(),
                    r.lon_b.to_string(),
                    r.lat_b,
                    r.lon_b,
                );

                process_geo(&mut writer, &geo, &mut id, cli.strict, &mut invalid)?;
            }
        }
    }

    writer.flush()?;

    if invalid > 0 {
        eprintln!("{} ignored line(s)", invalid);
    }

    Ok(())
}

// Build a fully normalized geo structure.
fn build_normalized_geo(
    name_a: String,
    lat_a_in: String,
    lon_a_in: String,
    lat_a_dd: f64,
    lon_a_dd: f64,
    name_b: String,
    lat_b_in: String,
    lon_b_in: String,
    lat_b_dd: f64,
    lon_b_dd: f64,
) -> NormalizedGeo {
    let lat_a_dd = round(lat_a_dd, 6);
    let lon_a_dd = round(lon_a_dd, 6);
    let lat_b_dd = round(lat_b_dd, 6);
    let lon_b_dd = round(lon_b_dd, 6);

    NormalizedGeo {
        a: NormalizedPoint {
            name: name_a,
            lat: NormalizedCoord {
                input: lat_a_in,
                dd: lat_a_dd,
                dms: dd_to_dms(lat_a_dd, CoordinateKind::Latitude),
            },
            lon: NormalizedCoord {
                input: lon_a_in,
                dd: lon_a_dd,
                dms: dd_to_dms(lon_a_dd, CoordinateKind::Longitude),
            },
        },
        b: NormalizedPoint {
            name: name_b,
            lat: NormalizedCoord {
                input: lat_b_in,
                dd: lat_b_dd,
                dms: dd_to_dms(lat_b_dd, CoordinateKind::Latitude),
            },
            lon: NormalizedCoord {
                input: lon_b_in,
                dd: lon_b_dd,
                dms: dd_to_dms(lon_b_dd, CoordinateKind::Longitude),
            },
        },
    }
}

// Process one normalized geo entry.
fn process_geo(
    writer: &mut Writer<File>,
    geo: &NormalizedGeo,
    id: &mut u64,
    strict: bool,
    invalid: &mut u64,
) -> Result<(), AppError> {

    // Compute distance.
    let dist_km = round(
        haversine(geo.a.lat.dd, geo.a.lon.dd, geo.b.lat.dd, geo.b.lon.dd,)?,
        2,
    );
    // Compute proximity comparison.
    let nearly = compute_nearly(
        geo.a.lat.dd,
        geo.a.lon.dd,
        geo.b.lat.dd,
        geo.b.lon.dd,
        GeoTolerance::DEFAULT,
    );

    let distance_metrics = DistanceMetrics {
        km: dist_km,
        miles: round(dist_km * KM_TO_MILES, 2),
        nearly: nearly,
    };

    // Write output row.
    if let Err(e) = write_output(writer, geo, &distance_metrics, *id) {
        if strict {
            return Err(e.into());
        }
        *invalid += 1;
        return Ok(());
    }

    *id += 1;
    Ok(())
}

// Serialize one CSV output row.
fn write_output(
    writer: &mut Writer<File>,
    geo: &NormalizedGeo,
    distance_metrics: &DistanceMetrics,
    id: u64,
) -> Result<(), csv::Error> {

    writer.serialize(OutputRecord {
        id,
        name_a: geo.a.name.clone(),
        lat_a_in: geo.a.lat.input.clone(),
        lon_a_in: geo.a.lon.input.clone(),
        lat_a_dd: geo.a.lat.dd,
        lon_a_dd: geo.a.lon.dd,
        lat_a_dms: geo.a.lat.dms.clone(),
        lon_a_dms: geo.a.lon.dms.clone(),
        name_b: geo.b.name.clone(),
        lat_b_in: geo.b.lat.input.clone(),
        lon_b_in: geo.b.lon.input.clone(),
        lat_b_dd: geo.b.lat.dd,
        lon_b_dd: geo.b.lon.dd,
        lat_b_dms: geo.b.lat.dms.clone(),
        lon_b_dms: geo.b.lon.dms.clone(),
        distance_km: distance_metrics.km,
        distance_miles: distance_metrics.miles,
        nearly_lat: distance_metrics.nearly.lat,
        nearly_lon: distance_metrics.nearly.lon,
        nearly_both: distance_metrics.nearly.both,
    })?;

    Ok(())
}

/* ---------------- TEST ---------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::CoordField;

    /* --- round() --------------------*/
    #[test]
    fn test_round_basic() {
        assert_eq!(round(1.23456, 2), 1.23);
        assert_eq!(round(1.23556, 2), 1.24);
        assert_eq!(round(-1.23456, 3), -1.235);
    }

    /* --- DMS --------------------*/
    #[test]
    fn test_dms_latitude() {
        let v = dms_to_dd("48°51'29\"N", CoordinateKind::Latitude).unwrap();
        let v = round(v, 6);
        assert_eq!(v, 48.858056);
    }

    #[test]
    fn test_dms_longitude_west() {
        let v = dms_to_dd("2°17'40\"W", CoordinateKind::Longitude).unwrap();
        let v = round(v, 6);
        assert_eq!(v, -2.294444);
    }

    #[test]
    fn test_dms_longitude_ouest_fr() {
        let v = dms_to_dd("2°17'40\"O", CoordinateKind::Longitude).unwrap();
        let v = round(v, 6);
        assert_eq!(v, -2.294444);
    }

    #[test]
    fn test_dms_invalid_direction() {
        assert!(dms_to_dd("48°51'29\"X", CoordinateKind::Latitude).is_err());
    }

    #[test]
    fn test_dms_invalid_format() {
        assert!(dms_to_dd("48.858056", CoordinateKind::Latitude).is_err());
    }

    #[test]
    fn test_dms_missing_deg_field() {
        assert!(dms_to_dd("°0'0\"N", CoordinateKind::Latitude).is_err());
    }

    #[test]
    fn test_dms_invalid_minutes_field() {
        assert!(dms_to_dd("48°V'0\"N", CoordinateKind::Latitude).is_err());
    }

    #[test]
    fn test_dms_invalid_seconds_field() {
        assert!(dms_to_dd("48°0'O\"N", CoordinateKind::Latitude).is_err());
    }

    #[test]
    fn test_dms_invalid_latitude_value() {
        assert!(matches!(
            dms_to_dd("91°0'0\"N", CoordinateKind::Latitude),
            Err(DmsError::InvalidCoord(_))
        ));
    }

    #[test]
    fn test_dms_invalid_minutes_value() {
        assert!(matches!(
            dms_to_dd("48°61'57\"N", CoordinateKind::Latitude),
            Err(DmsError::InvalidCoord(_))
        ));
    }

    #[test]
    fn test_dms_invalid_seconds_value() {
        assert!(matches!(
            dms_to_dd("48°61'57\"N", CoordinateKind::Latitude),
            Err(DmsError::InvalidCoord(_))
        ));
    }

    #[test]
    fn test_dms_unicode_symbols() {
        let v = dms_to_dd("48°51′29″N", CoordinateKind::Latitude).unwrap();
        let v = round(v, 6);
        assert_eq!(v, 48.858056);
    }

    #[test]
    fn test_dms_spaces() {
        let v = dms_to_dd("48° 51 ' 29\" N", CoordinateKind::Latitude).unwrap();
        let v = round(v, 6);
        assert_eq!(v, 48.858056);
    }

    #[test]
    fn test_dms_latitude_90_is_valid() {
        let v = dms_to_dd("90°0'0\"N", CoordinateKind::Latitude).unwrap();
        assert_eq!(v, 90.0);
    }

    #[test]
    fn test_dms_longitude_180_is_valid() {
        let v = dms_to_dd("180°0'0\"E", CoordinateKind::Longitude).unwrap();
        assert_eq!(v, 180.0);
    }

    #[test]
    fn test_dms_invalid_format_vs_invalid_value() {
        // Format invalid
        assert!(matches!(
            dms_to_dd("48.858056", CoordinateKind::Latitude),
            Err(DmsError::InvalidFormat)
        ));

        // Value invalid (minutes > 60)
        assert!(matches!(
            dms_to_dd("48°61'0\"N", CoordinateKind::Latitude),
            Err(DmsError::InvalidCoord(_))
        ));

        // Field invalid
        assert!(matches!(
            dms_to_dd("48c°57'0\"N", CoordinateKind::Latitude),
            Err(DmsError::InvalidField { field: CoordField::Deg })
        ));

    }

    /* --- DDM --------------------*/

    #[test]
    fn test_ddm_invalid_deg_field() {
        assert!(matches!(
            ddm_to_dd("48c°57'N", CoordinateKind::Latitude),
            Err(DdmError::InvalidField { field: CoordField::Deg })
        ));
    }

    #[test]
    fn test_ddm_invalid_minutes_value() {
        assert!(matches!(
            ddm_to_dd("48°61'N", CoordinateKind::Latitude),
            Err(DdmError::InvalidCoord(_))
        ));
    }

    #[test]
    fn test_ddm_invalid_format_vs_invalid_value() {
        // Format invalid
        assert!(matches!(
            ddm_to_dd("48.858056", CoordinateKind::Latitude),
            Err(DdmError::InvalidFormat)
        ));

        // Value invalid (minutes > 60)
        assert!(matches!(
            ddm_to_dd("48°61'N", CoordinateKind::Latitude),
            Err(DdmError::InvalidCoord(_))
        ));

        // Minutes negative
        assert!(matches!(
            ddm_to_dd("48°-1'N", CoordinateKind::Latitude),
            Err(DdmError::InvalidCoord(_))
        ));

        // Latitude out of boundaries
        assert!(matches!(
            ddm_to_dd("91°0'N", CoordinateKind::Latitude),
            Err(DdmError::InvalidCoord(_))
        ));

        // Direction invalid
        assert!(matches!(
            ddm_to_dd("48°30'X", CoordinateKind::Latitude),
            Err(DdmError::InvalidCoord(_))
        ));
    }

    #[test]
    fn test_ddm_to_distance_integration() -> Result<(), Box<dyn std::error::Error>> {

        let turing_eiffel_lat = "48° 51.492' N";
        let turing_eiffel_lon = "2° 17.652' E";

        let statue_liberty_lat = "40° 41.358' N";
        let statue_liberty_lon = "74° 2.646' W";

        let lat1 = ddm_to_dd(turing_eiffel_lat, CoordinateKind::Latitude)?;
        let lon1 = ddm_to_dd(turing_eiffel_lon, CoordinateKind::Longitude)?;

        let lat2 = ddm_to_dd(statue_liberty_lat, CoordinateKind::Latitude)?;
        let lon2 = ddm_to_dd(statue_liberty_lon, CoordinateKind::Longitude)?;

        assert!(lat1 > 0.0);
        assert!(lon1 > 0.0);
        assert!(lat2 > 0.0);
        assert!(lon2 < 0.0);

        let distance = haversine(lat1, lon1, lat2, lon2)?;

        let distance_rounded = round(distance, 2);

        assert!((distance_rounded - 5837.0).abs() < 5.0);

        Ok(())
    }

}
