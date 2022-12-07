mod adapters;
mod database;
mod domain;
mod resources;
mod services;
mod utils;

use crate::database::init_db;
use crate::resources::{feature_flags_api, environments_api, client_api};
use actix_web::{web, App, HttpServer, http};
use mongodb::Database;
use actix_cors::Cors;

struct AppState {
    app_name: String,
    db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = init_db().await.unwrap();
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:5173")
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://localhost")
            .allowed_origin("http://.*")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b"127.0.0.1:5173")
            })
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b"localhost:5173")
            })
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b"localhost")
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(AppState {
                app_name: String::from("Feature Flags"),
                db: db.clone(),
            }))
            .service(client_api::create_scope())
            .service(feature_flags_api::create_scope())
            .service(environments_api::create_scope())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
