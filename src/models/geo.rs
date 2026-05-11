// src/models/geo.rs

use crate::util::Nearly;

// Normalized coordinate representation.
#[derive(Debug, Clone)]
pub struct NormalizedCoord {
    pub input: String, // original input string
    pub dd: f64,       // decimal degrees
    pub dms: String,   // formatted DMS output
}

// Normalized geographic point.
#[derive(Debug, Clone)]
pub struct NormalizedPoint {
    pub name: String,
    pub lat: NormalizedCoord,
    pub lon: NormalizedCoord,
}

// Normalized geographic point.
#[derive(Debug, Clone)]
pub struct NormalizedGeo {
    pub a: NormalizedPoint,
    pub b: NormalizedPoint,
}

// Distance and comparison metrics.
#[derive(Debug)]
pub struct DistanceMetrics {
    pub km: f64,
    pub miles: f64,
    pub nearly: Nearly,
}
