// src/models/input.rs

use serde::Deserialize;

/* ---------------- INPUT CSV STRUCTS ---------------- */

// Decimal degrees input.
#[derive(Debug, Deserialize)]
pub struct InputDecimal {
    pub name_a: String,
    pub lat_a: f64,
    pub lon_a: f64,
    pub name_b: String,
    pub lat_b: f64,
    pub lon_b: f64,
}

// String-based input (DMS / DDM).
#[derive(Debug, Deserialize)]
pub struct InputString {
    pub name_a: String,
    pub lat_a: String,
    pub lon_a: String,
    pub name_b: String,
    pub lat_b: String,
    pub lon_b: String,
}
