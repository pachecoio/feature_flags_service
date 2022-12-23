mod adapters;
mod database;
mod domain;
mod resources;
mod services;
mod utils;

use std::collections::HashMap;
use std::sync::Mutex;
use crate::database::init_db;
use crate::resources::{feature_flags_api, environments_api, client_api};
use actix_web::{web, App, HttpServer, http};
use mongodb::Database;
use actix_cors::Cors;
use actix_web::web::Data;
use crate::domain::models::{Environment, FeatureFlag};

struct AppState {
    app_name: String,
    db: Database,
    flags: Vec<FeatureFlag>,
    envs: HashMap<String, Environment>
}

async fn get_state() -> Data<Mutex<AppState>> {
    Data::new(Mutex::new(AppState {
        app_name: String::from("Feature Flags"),
        db: init_db().await.unwrap(),
        flags: Vec::new(),
        envs: HashMap::new()
    }))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = get_state().await;
    HttpServer::new(move || {
        let cors = get_cors();
        App::new()
            .wrap(cors)
            .app_data(Data::clone(&app_data))
            .service(client_api::create_scope())
            .service(feature_flags_api::create_scope())
            .service(environments_api::create_scope())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

fn get_cors() -> Cors {
    Cors::default()
        .allowed_origin("http://127.0.0.1:5173")
        .allowed_origin("http://localhost:5173")
        .allowed_origin("http://localhost")
        .allowed_origin("http://.*")
        .allowed_origin("https://ff.pacheco.io")
        .allowed_origin_fn(|origin, _req_head| {
            origin.as_bytes().ends_with(b"127.0.0.1:5173")
        })
        .allowed_origin_fn(|origin, _req_head| {
            origin.as_bytes().ends_with(b"localhost:5173")
        })
        .allowed_origin_fn(|origin, _req_head| {
            origin.as_bytes().ends_with(b"localhost")
        })
        .allowed_origin_fn(|origin, _req_head| {
            origin.as_bytes().ends_with(b"ff.pacheco.io")
        })
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600)
}