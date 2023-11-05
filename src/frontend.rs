//! Frontend related functions.

use actix_files as fs;
use actix_web::{
    dev::ServiceResponse,
    get,
    web::{Redirect, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};

#[get("/")]
/// Route to redirect web root to index.html
async fn index() -> impl Responder {
    Redirect::to("/index.html").permanent()
}

/// Service handler for NotFound Response.
fn directory(_: &fs::Directory, req: &HttpRequest) -> Result<ServiceResponse, std::io::Error> {
    let response = HttpResponse::NotFound().finish();
    Ok(ServiceResponse::new(req.clone(), response))
}

/// Configuration function for the application.
pub fn frontend_cfg(cfg: &mut ServiceConfig) {
    cfg.service(index).service(
        fs::Files::new("/", "./frontend/src")
            .files_listing_renderer(directory)
            .show_files_listing(),
    );
}
