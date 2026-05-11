// src/pipeline.writer.rs

use csv::Writer;
use std::fs::File;

use crate::models::{
    geo::{DistanceMetrics, NormalizedGeo},
    output::OutputRecord,
};

// Serialize one CSV output row.
pub fn write_output(
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
