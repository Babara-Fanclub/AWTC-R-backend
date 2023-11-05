//! REST API related functions.

use actix_web::{
    error::InternalError,
    web::{scope, JsonConfig, ServiceConfig},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use self::{
    data::data_cfg, gps::gps_cfg, led_test::led_test_cfg, paths::paths_cfg, trips::trips_cfg,
};

pub use led_test::Colour;

mod data;
mod gps;
mod led_test;
mod paths;
mod trips;

/// Configuration function for the API resources.
pub fn api_cfg(cfg: &mut ServiceConfig) {
    let config = JsonConfig::default().error_handler(|err, _| {
        InternalError::from_response(
            err,
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Bad JSON Data"
            })),
        )
        .into()
    });
    cfg.service(
        scope("/api")
            .app_data(config)
            .configure(data_cfg)
            .configure(trips_cfg)
            .configure(gps_cfg)
            .configure(led_test_cfg)
            .configure(paths_cfg),
    );
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
/// A struct representing a coordinate in a map.
pub struct Coordinates {
    #[serde(alias = "lat")]
    /// The longitude of the coordinate.
    latitude: f64,
    #[serde(alias = "lng")]
    /// The lattitude of the coordinate.
    longitude: f64,
}
