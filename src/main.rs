//! Web Server for Group Project
//!
//! Copyright: ecyht2

mod api;
mod frontend;

use std::sync::Mutex;

use actix_web::web;
use actix_web::{web::ServiceConfig, HttpResponse, Responder};
use api::{api_cfg, Colour};
use frontend::frontend_cfg;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::CustomError;
use sqlx::{Executor, PgPool};

pub struct AppState {
    pub pool: PgPool,
    pub colour: Mutex<Colour>,
}

/// Service handler for NotFound Response.
async fn not_found() -> impl Responder {
    HttpResponse::NotFound().finish()
}

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_shared_db::Postgres(local_uri = "{secrets.DB}")] pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    // Creating tables
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let state = web::Data::new(AppState {
        pool,
        colour: Mutex::new(Colour::Red),
    });

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("")
                .configure(api_cfg)
                .configure(frontend_cfg)
                .app_data(state)
                .default_service(web::route().to(not_found)),
        );
    };

    Ok(config.into())
}
