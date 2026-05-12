// src/pipeline/normalized

use csv::Writer;
use std::fs::File;

use crate::{
    error::AppError,
    geo::coordinate::CoordinateKind,
    models::{
        geo::{DistanceMetrics, NormalizedCoord, NormalizedGeo, NormalizedPoint},
        output::OutputRecord,
    },
    util::{GeoTolerance, KM_TO_MILES, Nearly, haversine, round},
};

// Build a fully normalized geo structure.
#[allow(clippy::too_many_arguments)]
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

// Process one normalized geo entry.
pub fn process_geo(
    writer: &mut Writer<File>,
    geo: &NormalizedGeo,
    id: &mut u64,
    strict: bool,
    invalid: &mut usize,
) -> Result<(), AppError> {
    // Compute distance.
    let dist_km = round(haversine(geo.a.lat.dd, geo.a.lon.dd, geo.b.lat.dd, geo.b.lon.dd)?, 2);
    // Compute proximity comparison.
    let nearly = Nearly::compute_nearly(
        geo.a.lat.dd,
        geo.a.lon.dd,
        geo.b.lat.dd,
        geo.b.lon.dd,
        GeoTolerance::DEFAULT,
    );

    let distance_metrics = DistanceMetrics {
        km: dist_km,
        miles: round(dist_km * KM_TO_MILES, 2),
        nearly,
    };

    // Write output row.
    if let Err(e) = write_output(writer, geo, &distance_metrics, *id) {
        if strict {
            return Err(e.into());
        }
        *invalid += 1;
        return Ok(());
    }

    *id += 1;
    Ok(())
}

// Serialize one CSV output row.
fn write_output(
    writer: &mut Writer<File>,
    geo: &NormalizedGeo,
    distance_metrics: &DistanceMetrics,
    id: u64,
) -> Result<(), csv::Error> {
    writer.serialize(OutputRecord {
        id,
        name_a: geo.a.name.clone(),
        lat_a_in: geo.a.lat.input.clone(),
        lon_a_in: geo.a.lon.input.clone(),
        lat_a_dd: geo.a.lat.dd,
        lon_a_dd: geo.a.lon.dd,
        lat_a_dms: geo.a.lat.dms.clone(),
        lon_a_dms: geo.a.lon.dms.clone(),
        name_b: geo.b.name.clone(),
        lat_b_in: geo.b.lat.input.clone(),
        lon_b_in: geo.b.lon.input.clone(),
        lat_b_dd: geo.b.lat.dd,
        lon_b_dd: geo.b.lon.dd,
        lat_b_dms: geo.b.lat.dms.clone(),
        lon_b_dms: geo.b.lon.dms.clone(),
        distance_km: distance_metrics.km,
        distance_miles: distance_metrics.miles,
        nearly_lat: distance_metrics.nearly.lat,
        nearly_lon: distance_metrics.nearly.lon,
        nearly_both: distance_metrics.nearly.both,
    })?;

    Ok(())
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
