//! Module for Actix services for all the trips.

use actix_web::{
    error::ErrorBadRequest,
    get, post,
    web::{self, Data, Json, ServiceConfig},
    Responder, Result,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::AppState;

/// Configuration function for the trips API resources.
pub fn trips_cfg(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/trips").service(get_trips).service(start_trip));
}

#[derive(Serialize, FromRow)]
/// The data format for trips data.
struct TripValues {
    /// The UUID of the trip.
    uuid: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    /// When the trip started.
    time: OffsetDateTime,
    /// The path the trip is following.
    path: Uuid,
}

#[get("")]
/// Gets all the trips data.
async fn get_trips(state: Data<AppState>) -> Result<impl Responder> {
    let trips = sqlx::query_as!(TripValues, "SELECT uuid, time, path FROM trips")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ErrorBadRequest(e.to_string()))?;
    Ok(Json(trips))
}

#[derive(Serialize, FromRow)]
/// The reponse message for starting a new trip.
struct TripResponse {
    /// The UUID of the new trip.
    uuid: Uuid,
}

#[derive(Deserialize, FromRow)]
/// The input data format for inserting trip.
struct TripInput {
    /// The ptath the trip is following.
    path: Uuid,
}

#[post("")]
/// Starts a new trip.
async fn start_trip(trip: Json<TripInput>, state: Data<AppState>) -> Result<impl Responder> {
    let trip = sqlx::query_as!(
        TripResponse,
        "INSERT INTO trips (uuid, time, path) VALUES ($1, CURRENT_TIMESTAMP, $2) RETURNING uuid",
        Uuid::new_v4(),
        trip.path
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| ErrorBadRequest(e.to_string()))?;
    Ok(Json(trip))
}
