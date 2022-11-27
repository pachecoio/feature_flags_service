use actix_web::{HttpResponse, Scope, web};
use actix_web::web::Json;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::adapters::repositories::environment_repository::environment_repository_factory;
use crate::AppState;
use crate::domain::models::{Environment, FeatureFlag};
use crate::resources::CustomError;
use crate::services::environment_handlers;


async fn find(data: web::Data<AppState>) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = environment_repository_factory(db).await;
    let res = environment_handlers::find(&repo, None).await;
    let envs = match res {
        Ok(envs) => envs,
        Err(_) => vec![],
    };
    Ok(HttpResponse::Ok().json(EnvironmentList { items: envs }))
}

#[derive(Serialize, Deserialize)]
pub struct EnvironmentList {
    items: Vec<Environment>
}

async fn create(
    data: web::Data<AppState>,
    body: Json<Environment>,
) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = environment_repository_factory(db).await;
    match environment_handlers::create(&repo, &body.name).await {
        Ok(id) => {
            let mut env = Environment::new(&body.name);
            let env_id = ObjectId::parse_str(id).expect("");
            env.id = Some(env_id);
            Ok(HttpResponse::Created().json(Json(env)))
        }
        Err(_) => Err(CustomError::Conflict),
    }
}


async fn get(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = environment_repository_factory(db).await;
    let env_id = id.into_inner();
    match environment_handlers::get(&repo, &env_id).await {
        Ok(mut env) => {
            let _id = ObjectId::parse_str(env_id).unwrap();
            env.id = Some(_id);
            Ok(HttpResponse::Ok().json(Json(env)))
        }
        Err(_) => Err(CustomError::NotFound),
    }
}

async fn delete(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = environment_repository_factory(db).await;
    let flag_id = id.into_inner();
    match environment_handlers::delete(&repo, &flag_id).await {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(_) => Err(CustomError::NotFound),
    }
}


pub fn create_scope() -> Scope {
    web::scope("/environments")
        .route("", web::get().to(find))
        .route("/{id}", web::get().to(get))
        .route("", web::post().to(create))
        .route("/{id}", web::delete().to(delete))
}


#[cfg(test)]
mod tests {
    use std::process::id;
    use actix_web::{App, test};
    use actix_web::http::StatusCode;
    use mongodb::bson::doc;
    use serde_json::json;
    use crate::adapters::repositories::environment_repository::environment_repository_factory;
    use crate::AppState;
    use crate::database::init_db;
    use crate::domain::models::Environment;
    use super::*;

    #[actix_web::test]
    async fn test_find() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    app_name: String::from("Feature Flags"),
                    db: init_db().await.unwrap(),
                }))
                .service(create_scope()),
        )
        .await;
        let env = Environment::new("development");
        let req = test::TestRequest::get()
            .uri("/environments").to_request();
        let resp: EnvironmentList = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.items.len(), 0);
    }

    #[actix_web::test]
    async fn test_create_environment() {
        let db = init_db().await.unwrap();
        let repo = environment_repository_factory(&db).await;
        repo.collection.delete_many(doc! {}, None).await.unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    app_name: String::from("Feature Flags"),
                    db: db.clone(),
                }))
                .service(create_scope()),
        )
        .await;
        let env = Environment::new("development");
        let req = test::TestRequest::post()
            .uri("/environments")
            .set_json(Json(env)).to_request();
        let resp: Environment = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "development");

        let req = test::TestRequest::get()
            .uri(&format!("/environments/{}", resp.id.unwrap().to_string()))
            .to_request();
        let resp: Environment = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "development");

        let req = test::TestRequest::delete()
            .uri(&format!("/environments/{}", resp.id.unwrap()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }
}