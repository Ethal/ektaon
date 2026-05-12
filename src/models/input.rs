// src/models/input.rs

use serde::Deserialize;

/* ---------------- INPUT CSV STRUCTS ---------------- */

// String based input for coordinate parsing
#[derive(Debug, Clone, Deserialize)]
pub struct InputRow {
    pub row: usize,
    pub coordinate: RawCoordinate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCoordinate {
    pub name_a: String,
    pub lat_a: String,
    pub lon_a: String,
    pub name_b: String,
    pub lat_b: String,
    pub lon_b: String,
}
