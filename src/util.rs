//src/util.rs

use serde::{Serialize};

/* ---------------- CONSTANTES ---------------- */

// Overall numerical precision used for geographical comparisons.
const GEO_PRECISION: f64 = 1e-10;
// Average radius of the Earth in kilometers (spherical model).
const EARTH_RADIUS_KM: f64 = 6371.0;
// Conversion factor kilometers → miles.
pub const KM_TO_MILES: f64 = 0.621371;

/* ---------------- NUMERIC UTILS -------------- */

// Rounding of a floating-point number to N decimal places (max 10).
// Intentional limit to avoid excessively large exponents.
pub fn round(value: f64, decimals: u32) -> f64 {
    let precision = decimals.min(10);
    let factor = 10_f64.powi(precision as i32);
    (value * factor).round() / factor
}

/* ---------------- GEO DISTANCE--------------- */

// Errors specific to Haversine calculation.
#[derive(Debug, thiserror::Error)]
pub enum HaversineError {
    #[error("invalid distance")]
    InvalidDistance,

    // A negative distance should never happen.
    #[error("negative distance`{dist}`")]
    NegativeDistance { dist: f64 },
}

// Calculation of the great circle distance (Haversine).
// Inputs in decimal degrees.
// Output in kilometers.
pub fn haversine(lat1_deg: f64, lon1_deg: f64, lat2_deg: f64, lon2_deg: f64) -> Result<f64, HaversineError> {

    // Conversion degrés → radians
    let lat1 = lat1_deg.to_radians();
    let lon1 = lon1_deg.to_radians();
    let lat2 = lat2_deg.to_radians();
    let lon2 = lon2_deg.to_radians();

    // Angular differences.
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    // Haversine formula.
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);

    let distance = 2.0 * EARTH_RADIUS_KM * a.sqrt().atan2((1.0 - a).sqrt());

    // Security checks of the calculation.
    if !distance.is_finite() {
        return Err(HaversineError::InvalidDistance);
    }
    if distance < -GEO_PRECISION {
        return Err(HaversineError::NegativeDistance {dist: distance});
    }

    Ok(distance)
}

/* ---------------- GEO COMPARISON --------------- */

// Tolerance expressed in decimal degrees.
// Used to compare two coordinates with a margin of error.
#[derive(Debug, Copy, Clone)]
pub struct GeoTolerance {
    pub deg: f64,
}

impl GeoTolerance {
    // Default tolerance (~11 cm at the equator).
    pub const DEFAULT: Self = Self { deg: 1e-6 };
}

// Simple comparison of two angular values.
fn nearly_equal_deg(a: f64, b: f64, tol: GeoTolerance) -> bool {
    (a - b).abs() <= tol.deg
}

// Structured result of geographical comparison.
#[derive(Debug, Serialize)]
pub struct Nearly {
    pub lat: bool,
    pub lon: bool,
    pub both: bool,
}

// Compare two geographical positions with a given tolerance.
// Each axis is evaluated independently.
pub fn compute_nearly(
    lat_a: f64,
    lon_a: f64,
    lat_b: f64,
    lon_b: f64,
    tol: GeoTolerance,
) -> Nearly {
    let lat = nearly_equal_deg(lat_a, lat_b, tol);
    let lon = nearly_equal_deg(lon_a, lon_b, tol);

    Nearly {
        lat,
        lon,
        both: lat && lon,
    }
}
