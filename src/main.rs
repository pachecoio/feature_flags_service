mod domain;
mod resources;
mod database;
mod adapters;
mod services;
use actix_web::{web, App, HttpServer, Responder};
use mongodb::Database;
use crate::database::init_db;
use crate::domain::models::FeatureFlag;
use crate::resources::feature_flags_api;

struct AppState {
    app_name: String,
    db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = init_db().await.unwrap();
    HttpServer::new(move || {
        App::new().app_data(web::Data::new(AppState {
            app_name: String::from("Feature Flags"),
            db: db.clone()
        })).service(feature_flags_api::create_scope())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
