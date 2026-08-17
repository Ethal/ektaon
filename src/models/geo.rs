// src/models/geo.rs

use serde::{Deserialize, Serialize};

use crate::util::Nearly;

/* --------------- Project Normalized -----------------*/

// Normalized coordinate representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct NormalizedCoord {
    pub input: String, // original input string
    pub dd: f64,       // decimal degrees
    pub dms: String,   // formatted DMS output
}

// Normalized geographic point.
#[derive(Debug, Serialize, Deserialize)]
pub struct NormalizedPoint {
    pub name: String,
    pub lat: NormalizedCoord,
    pub lon: NormalizedCoord,
}

// Normalized geographic point.
#[derive(Debug, Serialize, Deserialize)]
pub struct NormalizedGeo {
    pub a: NormalizedPoint,
    pub b: NormalizedPoint,
}

// Distance and comparison metrics.
#[derive(Debug, Serialize, Deserialize)]
pub struct DistanceMetrics {
    pub km: f64,
    pub miles: f64,
    pub nearly: Nearly,
}

/* --------------- GeoJson  RFC7946 -----------------*/

/// GeoJSON root object
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GeoJson {
    FeatureCollection(FeatureCollection),
    Feature(Feature),
    Geometry(Geometry),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureCollection {
    pub features: Vec<Feature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,

    pub geometry: Option<Geometry>,
}

// [longitude, latitude]
type Position = (f64, f64);

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "coordinates")]
pub enum Geometry {
    Point(Position),
    LineString(Vec<Position>),
    Polygon(Vec<Vec<Position>>),

    MultiPoint(Vec<Position>),
    MultiLineString(Vec<Vec<Position>>),
    MultiPolygon(Vec<Vec<Vec<Position>>>),

    Collection(Vec<Geometry>),
}
