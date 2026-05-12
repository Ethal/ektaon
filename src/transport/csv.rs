// src/transport/csv.rs

use csv::{Reader, ReaderBuilder};
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use crate::error::AppError;
use crate::models::input::{InputRow, RawCoordinate};

// Required CSV headers (order-independent).
const REQUIRED_HEADERS: &[&str] = &["name_a", "lat_a", "lon_a", "name_b", "lat_b", "lon_b"];

fn header_validator(reader: &mut Reader<File>) -> Result<(), AppError> {
    // Validate required headers.
    let headers = reader.headers().map_err(|_| AppError::InvalidHeader)?;

    let headers: HashSet<_> = headers.iter().collect();
    for &h in REQUIRED_HEADERS {
        if !headers.contains(h) {
            return Err(AppError::MissingHeaderField(h.to_string()));
        }
    }
    Ok(())
}

//name_a,lat_a,lon_a, name_b,lat_b,lon_b
pub fn load_csv_rows(path: &Path, strict: bool) -> Result<(usize, Vec<InputRow>), AppError> {
    let mut coords = Vec::new();
    // CSV reader
    let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?; // one line is consume in the reader
    header_validator(&mut reader)?;

    // Processing counters.
    let mut invalid: usize = 0;
    let mut line_no = 1; // one line is consume in the reader for the header_validator.

    for row in reader.deserialize::<RawCoordinate>() {
        line_no += 1;
        let r = match row {
            Ok(v) => v,
            Err(_) => {
                invalid += 1;
                if strict {
                    return Err(AppError::InvalidRow { line: line_no });
                }
                continue;
            }
        };
        let coord = InputRow {
            row: line_no,
            coordinate: r,
        };
        coords.push(coord);
    }

    Ok((invalid, coords))
}
