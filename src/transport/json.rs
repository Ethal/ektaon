// src/transport/json.rs

use std::path::Path;

use crate::error::AppError;
use crate::models::input::{InputRow, RawCoordinate};

use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn load_jsonl(path: &Path, strict: bool) -> Result<(usize, Vec<InputRow>), AppError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut coords = Vec::new();
    let mut line_no = 0;
    let mut invalid = 0;

    for line in reader.lines() {
        line_no += 1;

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
