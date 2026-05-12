// srrc/pipeline/parser.rs

use crate::{
    error::{DdError, DdmError, DmsError},
    geo::{coordinate::CoordinateKind, dd::dd_to_dd, ddm::ddm_to_dd, dms::dms_to_dd},
    models::input::RawCoordinate,
};

pub fn parse_dms_row(r: &RawCoordinate) -> Result<(f64, f64, f64, f64), DmsError> {
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

pub fn parse_ddm_row(r: &RawCoordinate) -> Result<(f64, f64, f64, f64), DdmError> {
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

pub fn parse_dd_row(r: &RawCoordinate) -> Result<(f64, f64, f64, f64), DdError> {
    match (
        dd_to_dd(&r.lat_a, CoordinateKind::Latitude),
        dd_to_dd(&r.lon_a, CoordinateKind::Longitude),
        dd_to_dd(&r.lat_b, CoordinateKind::Latitude),
        dd_to_dd(&r.lon_b, CoordinateKind::Longitude),
    ) {
        (Ok(a), Ok(b), Ok(c), Ok(d)) => Ok((a, b, c, d)),

        (Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) | (_, _, _, Err(e)) => Err(e),
    }
}
