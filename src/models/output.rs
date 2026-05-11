// src/models/output.rs

use serde::Serialize;

/* ---------------- OUTPUT CSV STRUCTS ---------------- */

// Output CSV record (fully normalized).
#[derive(Debug, Serialize)]
pub struct OutputRecord {
    pub id: u64,

    pub name_a: String,
    pub lat_a_in: String,
    pub lon_a_in: String,
    pub lat_a_dd: f64,
    pub lon_a_dd: f64,
    pub lat_a_dms: String,
    pub lon_a_dms: String,

    pub name_b: String,
    pub lat_b_in: String,
    pub lon_b_in: String,
    pub lat_b_dd: f64,
    pub lon_b_dd: f64,
    pub lat_b_dms: String,
    pub lon_b_dms: String,

    pub distance_km: f64,
    pub distance_miles: f64,
    pub nearly_lat: bool,
    pub nearly_lon: bool,
    pub nearly_both: bool,
}
