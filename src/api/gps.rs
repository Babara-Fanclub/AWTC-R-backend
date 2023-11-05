//! Module for Actix services for all the history.
use actix_web::{
    error::{ErrorBadRequest, Result},
    get, post,
    web::{scope, Data, Json, Query, ServiceConfig},
    Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::AppState;

use super::Coordinates;

/// Configuration function for the gps API resources.
pub fn gps_cfg(cfg: &mut ServiceConfig) {
    cfg.service(scope("/gps").service(get_gps).service(add_gps));
}

#[derive(Deserialize)]
/// The query specification for getting gps data.
struct GPSQuery {
    #[serde(default = "GPSQuery::count_default")]
    /// The amount of data to get.
    count: i64,
}

impl GPSQuery {
    /// Defaults count to 100 coordinates.
    fn count_default() -> i64 {
        100
    }
}

#[derive(Serialize, FromRow)]
/// The data format for gps data.
struct GPSValues {
    /// The coordinate of the data.
    location: sqlx::types::JsonValue,
    #[serde(with = "time::serde::rfc3339")]
    /// The when the data is recorded.
    time: OffsetDateTime,
}

#[derive(Serialize, FromRow)]
/// The data format for gps data.
struct GPSOutput {
    /// The coordinate of the data.
    location: Coordinates,
    #[serde(with = "time::serde::rfc3339")]
    /// The when the data is recorded.
    time: OffsetDateTime,
}

impl TryFrom<GPSValues> for GPSOutput {
    type Error = serde_json::Error;

    fn try_from(value: GPSValues) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            location: serde_json::from_value(value.location)?,
            time: value.time,
        })
    }
}

#[get("")]
/// Gets the gps data from the database.
async fn get_gps(query: Query<GPSQuery>, state: Data<AppState>) -> Result<impl Responder> {
    let locations: Vec<GPSOutput> = sqlx::query_as!(
        GPSValues,
        "SELECT location, time FROM history ORDER BY time DESC LIMIT $1",
        query.count
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ErrorBadRequest(e.to_string()))?
    .into_iter()
    .map(|v| GPSOutput::try_from(v))
    .collect::<Result<Vec<_>, serde_json::Error>>()?;
    Ok(Json(locations))
}

#[post("")]
/// Create the gps data to the database.
async fn add_gps(data: Json<Coordinates>, state: Data<AppState>) -> Result<impl Responder> {
    sqlx::query!(
        "INSERT INTO history (location, time) VALUES ($1, CURRENT_TIMESTAMP)",
        serde_json::json!(data.0)
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ErrorBadRequest(e.to_string()))?;
    Ok("")
}
