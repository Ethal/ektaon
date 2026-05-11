// srrc/pipeline/parser.rs

use crate::{
    error::{DdmError, DmsError},
    geo::{coordinate::CoordinateKind, ddm::ddm_to_dd, dms::dms_to_dd},
    models::input::InputString,
};

pub fn parse_dms_row(r: &InputString) -> Result<(f64, f64, f64, f64), DmsError> {
    match (
        dms_to_dd(&r.lat_a, CoordinateKind::Latitude),
        dms_to_dd(&r.lon_a, CoordinateKind::Longitude),
        dms_to_dd(&r.lat_b, CoordinateKind::Latitude),
        dms_to_dd(&r.lon_b, CoordinateKind::Longitude),
    ) {
        (Ok(a), Ok(b), Ok(c), Ok(d)) => Ok((a, b, c, d)),

        (Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) | (_, _, _, Err(e)) => Err(e),
    }
}

pub fn parse_ddm_row(r: &InputString) -> Result<(f64, f64, f64, f64), DdmError> {
    match (
        ddm_to_dd(&r.lat_a, CoordinateKind::Latitude),
        ddm_to_dd(&r.lon_a, CoordinateKind::Longitude),
        ddm_to_dd(&r.lat_b, CoordinateKind::Latitude),
        ddm_to_dd(&r.lon_b, CoordinateKind::Longitude),
    ) {
        (Ok(a), Ok(b), Ok(c), Ok(d)) => Ok((a, b, c, d)),

        (Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) | (_, _, _, Err(e)) => Err(e),
    }
}
