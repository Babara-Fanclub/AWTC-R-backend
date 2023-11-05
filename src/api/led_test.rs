//! Module for testing API.
use actix_web::{
    get, post,
    web::{scope, Data, Json, ServiceConfig},
    Responder,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
/// An enumeration of all possible colours.
pub enum Colour {
    /// Red colour.
    Red,
    /// Green colour.
    Green,
    /// Blue colour.
    Blue,
}

/// Configuration function for the led_test API resources.
pub fn led_test_cfg(cfg: &mut ServiceConfig) {
    cfg.service(scope("/led_test").service(get_colour).service(set_colour));
}

#[derive(Serialize, Deserialize, Debug)]
/// The input JSON body for setting colour.
struct ColourJson {
    #[serde(alias = "color")]
    /// The colour to set to.
    colour: Colour,
}

#[get("")]
/// Gets the current colour.
async fn get_colour(data: Data<AppState>) -> impl Responder {
    let colour = data.colour.lock().unwrap();
    Json(ColourJson {
        colour: colour.clone(),
    })
}

#[post("")]
/// Sets the colour.
async fn set_colour(colour: Json<ColourJson>, data: Data<AppState>) -> impl Responder {
    let mut colour_data = data.colour.lock().unwrap();
    *colour_data = (*colour).colour.clone();
    ""
}
