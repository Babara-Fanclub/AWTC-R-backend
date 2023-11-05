//! Module for Actix services for all the paths.

use actix_web::{
    error::ErrorBadRequest,
    get, post,
    web::{self, Data, Json, Path, ServiceConfig},
    Responder, Result,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::AppState;

use super::Coordinates;

/// Configuration function for the paths API resources.
pub fn paths_cfg(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/paths")
            .service(get_paths)
            .service(register_path)
            .service(get_path),
    );
}

#[derive(Serialize, FromRow)]
/// The data format for trips data.
struct PathValues {
    /// The name of the path.
    name: String,
    /// The UUID of the path.
    uuid: Uuid,
}

#[get("")]
/// Gets all the trips data.
async fn get_paths(state: Data<AppState>) -> Result<impl Responder> {
    let paths = sqlx::query_as!(PathValues, "SELECT name, uuid FROM paths")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ErrorBadRequest(e.to_string()))?;
    Ok(Json(paths))
}

#[derive(FromRow, Debug)]
/// The data format for trip data.
struct PathData {
    /// The name of the path.
    name: String,
    /// The the points on the path.
    path: serde_json::Value,
}

#[derive(Serialize, FromRow, Debug)]
/// The data format for trip data.
struct PathDataCoords {
    /// The name of the path.
    name: String,
    /// The the points on the path.
    path: Vec<Coordinates>,
}

impl TryFrom<PathData> for PathDataCoords {
    type Error = serde_json::Error;

    fn try_from(value: PathData) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            path: serde_json::from_value(value.path)?,
        })
    }
}

#[get("{uuid}")]
/// Gets the list of coordinates for a path.
async fn get_path(uuid: Path<Uuid>, state: Data<AppState>) -> Result<impl Responder> {
    let paths = sqlx::query_as!(
        PathData,
        "SELECT name, path FROM paths WHERE uuid = $1",
        *uuid
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| ErrorBadRequest(e.to_string()))?;
    let paths = PathDataCoords::try_from(paths)?;
    Ok(Json(paths))
}

#[derive(Serialize, FromRow)]
/// The reponse message for starting a new trip.
struct PathResponse {
    #[serde(rename = "uuid")]
    /// The UUID of the new path.
    uuid: Uuid,
}

#[derive(Deserialize)]
/// The input data format for inserting path.
struct PathInput {
    /// The name of the path.
    name: String,
    /// The list of coordinates to follow.
    path: Vec<Coordinates>,
}

#[post("")]
/// Register a new path.
async fn register_path(path: Json<PathInput>, state: Data<AppState>) -> Result<impl Responder> {
    let paths: Vec<serde_json::Value> = path.path.iter().map(|v| serde_json::json!(v)).collect();
    let path_id = sqlx::query_as!(
        PathResponse,
        "INSERT INTO paths (name, path, uuid) VALUES ($1, $2, $3) RETURNING uuid",
        path.name,
        &paths,
        Uuid::new_v4()
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| ErrorBadRequest(e.to_string()))?;
    Ok(Json(path_id))
}
