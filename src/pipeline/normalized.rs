// src/pipeline/normalized

use crate::geo::coordinate::CoordinateKind;
use crate::models::geo::{NormalizedCoord, NormalizedGeo, NormalizedPoint};
use crate::util::round;

// Build a fully normalized geo structure.
pub fn build_normalized_geo(
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

/* ---------------- FORMATTING ---------------- */

// Converts decimal degrees to a DMS string.
// This function does not perform validation.
fn dd_to_dms(value: f64, kind: CoordinateKind) -> String {
    let dir = if kind == CoordinateKind::Latitude {
        if value >= 0.0 { 'N' } else { 'S' }
    } else {
        if value >= 0.0 { 'E' } else { 'W' }
    };

    let abs = value.abs();
    let deg = abs.floor();
    let min_f = (abs - deg) * 60.0;
    let min = min_f.floor();
    let sec = (min_f - min) * 60.0;

    format!("{}°{}'{:.2}\"{}", deg as i32, min as i32, sec, dir)
}
