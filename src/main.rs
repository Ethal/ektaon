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
use clap::Parser;
use csv::Writer;
use std::fs::File;

mod cli;
mod error;
mod geo;
mod models;
mod pipeline;
mod transport;
mod util;

use crate::{
    error::AppError,
    pipeline::{
        parser::{parse_dd_row, parse_ddm_row, parse_dms_row},
        process::{build_normalized_geo, process_geo},
    },
    transport::{
        csv::load_csv_rows,
        json::{load_json, load_jsonl},
    },
};

/* ---------------- MAIN ---------------- */

fn main() -> Result<(), AppError> {
    // Parse CLI arguments.
    let cli = cli::Cli::parse();

    // CSV writer setup for output.
    let mut writer = Writer::from_writer(File::create(cli.output)?);

    // Parsing as per input format
    let (invalid_parse, input_rows) = match cli.input_format {
        cli::InputFormat::Csv => load_csv_rows(&cli.input, cli.strict)?,
        cli::InputFormat::Json => load_json(&cli.input)?,
        cli::InputFormat::Jsonl => load_jsonl(&cli.input, cli.strict)?,
    };

    // Processing counters.
    let mut id: u64 = 1;
    let mut invalid: usize = invalid_parse;
    let mut line_no: usize;

    // Dispatch based on input format.
    match cli.coord_format {
        cli::CoordinateFormat::Dms => {
            for row in input_rows {
                line_no = row.row;
                let coord = row.coordinate;
                // Parse DMS coordinates.
                let (lat_a_dd, lon_a_dd, lat_b_dd, lon_b_dd) = match parse_dms_row(&coord) {
                    Ok(v) => v,
                    Err(e) => {
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
                    coord.name_a,
                    coord.lat_a,
                    coord.lon_a,
                    lat_a_dd,
                    lon_a_dd,
                    coord.name_b,
                    coord.lat_b,
                    coord.lon_b,
                    lat_b_dd,
                    lon_b_dd,
                );

                process_geo(&mut writer, &geo, &mut id, cli.strict, &mut invalid)?;
            }
        }
        cli::CoordinateFormat::Ddm => {
            for row in input_rows {
                line_no = row.row;
                let coord = row.coordinate;
                // Parse DDM coordinates.
                let (lat_a_dd, lon_a_dd, lat_b_dd, lon_b_dd) = match parse_ddm_row(&coord) {
                    Ok(v) => v,
                    Err(e) => {
                        invalid += 1;
                        if cli.strict {
                            return Err(AppError::InvalidDdm {
                                line: line_no,
                                source: e,
                            });
                        }
                        continue;
                    }
                };

                let geo = build_normalized_geo(
                    coord.name_a,
                    coord.lat_a,
                    coord.lon_a,
                    lat_a_dd,
                    lon_a_dd,
                    coord.name_b.clone(),
                    coord.lat_b,
                    coord.lon_b,
                    lat_b_dd,
                    lon_b_dd,
                );

                process_geo(&mut writer, &geo, &mut id, cli.strict, &mut invalid)?;
            }
        }
        cli::CoordinateFormat::Dd => {
            for row in input_rows {
                line_no = row.row;
                let coord = row.coordinate;
                // Parse DD coordinates.
                let (lat_a_dd, lon_a_dd, lat_b_dd, lon_b_dd) = match parse_dd_row(&coord) {
                    Ok(v) => v,
                    Err(e) => {
                        invalid += 1;
                        if cli.strict {
                            return Err(AppError::InvalidDd {
                                line: line_no,
                                source: e,
                            });
                        }
                        continue;
                    }
                };

                let geo = build_normalized_geo(
                    coord.name_a,
                    coord.lat_a.to_string(),
                    coord.lon_a.to_string(),
                    lat_a_dd,
                    lon_a_dd,
                    coord.name_b,
                    coord.lat_b.to_string(),
                    coord.lon_b.to_string(),
                    lat_b_dd,
                    lon_b_dd,
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

/* ---------------- TEST ---------------- */

#[cfg(test)]
mod tests {
    use crate::error::{DdmError, DmsError};
    use crate::{
        geo::{
            coordinate::{CoordField, CoordinateKind},
            ddm::ddm_to_dd,
            dms::dms_to_dd,
        },
        util::{haversine, round},
    };

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
