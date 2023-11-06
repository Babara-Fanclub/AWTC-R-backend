//! Module for Actix services for collected data.

use actix_web::{
    error::ErrorInternalServerError,
    get, post,
    web::{scope, Data, Json, Path, Query, ServiceConfig},
    HttpResponse, Responder, Result,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::{OffsetDateTime, UtcOffset};
use uuid::Uuid;

use crate::AppState;

use super::Coordinates;

/// Configuration function for the data API resources.
pub fn data_cfg(cfg: &mut ServiceConfig) {
    cfg.service(scope("/data").service(get_data).service(post_data));
}

#[derive(Deserialize, Debug)]
/// The type of format to respond with.
enum FormatType {
    #[serde(
        alias = "csv",
        alias = "Csv",
        alias = "cSv",
        alias = "csV",
        alias = "CSv",
        alias = "cSV",
        alias = "CsV"
    )]
    /// The response would be in CSV format.
    CSV,
    #[serde(other)]
    /// The response would be in JSON format.
    JSON,
}

impl Default for FormatType {
    /// FormatType defaults to JSON.
    fn default() -> Self {
        FormatType::JSON
    }
}

time::serde::format_description!(
    query_offset,
    UtcOffset,
    "[offset_hour padding:none]"
);

#[derive(Deserialize)]
/// The query specification for getting data.
struct DataQuery {
    #[serde(default)]
    /// The query to specify the format.
    format: FormatType,
    #[serde(default = "DataQuery::offset_default", with = "query_offset")]
    /// The query to specify the timezone offset for exported CSV.
    offset: UtcOffset,
}

impl DataQuery {
    fn offset_default() -> UtcOffset {
        UtcOffset::UTC
    }
}

#[derive(Deserialize)]
/// The trip specification for getting and adding data.
struct DataPath {
    /// The trip UUID the data belongs to.
    uuid: Uuid,
}

#[derive(Serialize, Debug, FromRow)]
/// The data format for data
struct DataValues {
    /// The temperature measured.
    temperature: f64,
    /// The location the data is measured.
    location: serde_json::Value,
    /// The depth the data is measured.
    depth: f64,
    /// The layer the data is measured.
    layer: Layer,
    #[serde(with = "time::serde::rfc3339")]
    /// The time the data is measured.
    time: OffsetDateTime,
}

#[derive(Serialize, Debug, FromRow)]
/// The data format for data
struct DataValuesOutput {
    /// The temperature measured.
    temperature: f64,
    /// The location the data is measured.
    location: serde_json::Value,
    /// The depth the data is measured.
    depth: f64,
    /// The layer the data is measured.
    layer: Layer,
    #[serde(with = "time::serde::rfc3339")]
    /// The time the data is measured.
    time: OffsetDateTime,
}

impl TryFrom<DataValues> for DataValuesOutput {
    fn try_from(value: DataValues) -> Result<Self, Self::Error> {
        Ok(DataValuesOutput {
            temperature: value.temperature,
            depth: value.depth,
            layer: value.layer,
            time: value.time,
            location: serde_json::from_value(value.location)?,
        })
    }

    type Error = serde_json::Error;
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "layer")]
#[sqlx(rename_all = "lowercase")]
/// Enumerations for all the water body layers/levels.
enum Layer {
    #[serde(rename = "surface")]
    /// The surface of the water body.
    Surface,
    #[serde(rename = "middle")]
    /// The middle of the water body.
    Middle,
    #[serde(rename = "sea bed")]
    #[sqlx(rename = "sea bed")]
    /// The sea bed of the water body.
    SeaBed,
}

time::serde::format_description!(
    csv_format,
    OffsetDateTime,
    "[day]/[month]/[year] [hour]:[minute]:[second] +[offset_hour]"
);

#[derive(Serialize, Debug, FromRow)]
/// The data format for data for CSV output
struct DataRecord {
    /// The temperature measured.
    temperature: f64,
    /// The location the data is measured.
    location: serde_json::Value,
    /// The depth the data is measured.
    depth: f64,
    /// The name of the path the data is collected on
    name: String,
    /// The layer the data is measured.
    layer: Layer,
    #[serde(with = "csv_format")]
    /// The time the data is measured.
    time: OffsetDateTime,
}

#[derive(Serialize, Debug, FromRow)]
/// The data format for data for CSV output
struct DataRecordOutput {
    /// The temperature measured.
    temperature: f64,
    /// The latitude the data is measured.
    latitude: f64,
    /// The longitude the data is measured.
    longitude: f64,
    /// The depth the data is measured.
    depth: f64,
    /// The name of the path the data is collected on
    name: String,
    /// The layer the data is measured.
    layer: Layer,
    #[serde(with = "csv_format")]
    /// The time the data is measured.
    time: OffsetDateTime,
}

impl TryFrom<DataRecord> for DataRecordOutput {
    fn try_from(value: DataRecord) -> Result<Self, Self::Error> {
        let location: Coordinates = serde_json::from_value(value.location)?;
        Ok(DataRecordOutput {
            temperature: value.temperature,
            depth: value.depth,
            layer: value.layer,
            time: value.time,
            name: value.name,
            latitude: location.latitude,
            longitude: location.longitude,
        })
    }

    type Error = serde_json::Error;
}

#[get("/{uuid}")]
/// Gets the data from the database.
async fn get_data(
    query: Query<DataQuery>,
    trip: Path<DataPath>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    Ok(match query.format {
        FormatType::CSV => get_csv(trip.uuid, query.offset, state).await?,
        _ => get_json(trip.uuid, state).await?,
    })
}

async fn get_json(trip: Uuid, state: Data<AppState>) -> Result<HttpResponse> {
    let data = sqlx::query_as!(
        DataValues,
        r#"SELECT data.temperature, data.location, data.depth, data.layer AS "layer: Layer",
 data.time FROM data WHERE data.trip = $1"#,
        trip
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?
    .into_iter()
    .map(|v| DataValuesOutput::try_from(v))
    .collect::<Result<Vec<_>, serde_json::Error>>()?;
    Ok(HttpResponse::Ok().json(data))
}

async fn get_csv(trip: Uuid, offset: UtcOffset, state: Data<AppState>) -> Result<HttpResponse> {
    let data = sqlx::query_as!(
        DataRecord,
        r#"SELECT data.temperature, data.location, data.depth, data.layer AS "layer: Layer",
 data.time, paths.name
FROM data
JOIN trips ON trips.uuid = data.trip
JOIN paths ON trips.path = paths.uuid
WHERE data.trip = $1"#,
        trip
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ErrorInternalServerError("An Error Occured When Fetching Data."))?;

    let mut writer = csv::Writer::from_writer(vec![]);
    // Adding header if there is no data
    if data.len() == 0 {
        writer
            .write_record(&[
                "temperature",
                "latitude",
                "longitude",
                "depth",
                "layer",
                "trip",
                "time",
            ])
            .unwrap();
    }
    // Inserting data
    for mut item in data {
        item.time = item.time.to_offset(offset);
        writer
            .serialize(DataRecordOutput::try_from(item)?)
            .map_err(|_| ErrorInternalServerError("An Error Occured When Fetching Data."))?
    }
    // Converting to string
    let csv = String::from_utf8(
        writer
            .into_inner()
            .map_err(|_| ErrorInternalServerError("An Error Occured When Fetching Data."))?,
    )
    .map_err(|_| ErrorInternalServerError("An Error Occured When Fetching Data."))?;
    // Sending CSV
    Ok(HttpResponse::Ok().content_type("text/csv").body(csv))
}

#[derive(Deserialize, FromRow)]
/// The input data format for inserting data
struct DataInput {
    /// The temperature measured.
    temperature: f64,
    /// The location the data is measured.
    location: Coordinates,
    /// The depth the data is measured.
    depth: f64,
    /// The layer the data is measured.
    layer: Layer,
    /// The trip the data is collected in.
    trip: Uuid,
}

#[post("")]
/// Insert new data to the database.
async fn post_data(data: Json<DataInput>, state: Data<AppState>) -> Result<impl Responder> {
    sqlx::query!(
        "INSERT INTO data (temperature, location, depth, layer, trip, time)
VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)",
        data.temperature,
        serde_json::json!(data.location),
        data.depth,
        data.layer.clone() as Layer,
        data.trip
    )
    .execute(&state.pool)
    .await
    .map_err(|_| actix_web::error::ErrorBadRequest("Invalid Data"))?;
    Ok("")
}
