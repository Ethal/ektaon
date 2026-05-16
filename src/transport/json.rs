// src/transport/json.rs

use std::fs;
use std::path::Path;

use crate::error::AppError;
use crate::models::geo::GeoJson;
use crate::models::input::{InputRow, RawCoordinate};

use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn load_jsonl(path: &Path, strict: bool) -> Result<(usize, Vec<InputRow>), AppError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut coords = Vec::new();
    let mut invalid = 0;

    for (line_no, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => {
                invalid += 1;
                if strict {
                    return Err(AppError::InvalidRow { line: line_no });
                }
                continue;
            }
        };

        let r: RawCoordinate = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                invalid += 1;
                if strict {
                    return Err(AppError::InvalidJsonRow { line: line_no });
                }
                continue;
            }
        };

        coords.push(InputRow {
            row: line_no,
            coordinate: r,
        });
    }

    Ok((invalid, coords))
}

pub fn load_json(path: &Path) -> Result<(usize, Vec<InputRow>), AppError> {
    let file = File::open(path)?;

    let rows: Vec<RawCoordinate> = serde_json::from_reader(file)?;

    let mut coords = Vec::new();

    for (i, r) in rows.into_iter().enumerate() {
        coords.push(InputRow {
            row: i + 1,
            coordinate: r,
        });
    }

    Ok((0, coords))
}

pub fn export_json(datas: &GeoJson, path: &Path) -> Result<(), AppError> {
    let json = serde_json::to_string_pretty(&datas)?;
    fs::write(path, json.as_bytes())?;

    Ok(())
}
