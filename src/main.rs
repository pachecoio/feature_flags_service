mod domain;
mod resources;
mod database;
mod adapters;
mod services;
use actix_web::{web, App, HttpServer, Responder};
use crate::resources::feature_flags_api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().app_data(
            AppSettings {}
        ).service(feature_flags_api::create_scope())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

pub struct AppSettings {}