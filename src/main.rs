mod adapters;
mod database;
mod domain;
mod resources;
mod services;
use crate::database::init_db;
use crate::resources::feature_flags_api;
use actix_web::{web, App, HttpServer};
use mongodb::Database;

struct AppState {
    app_name: String,
    db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = init_db().await.unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                app_name: String::from("Feature Flags"),
                db: db.clone(),
            }))
            .service(feature_flags_api::create_scope())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
