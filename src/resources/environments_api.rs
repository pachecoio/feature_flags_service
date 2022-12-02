use std::future::Future;
use actix_web::{HttpResponse, Scope, web};
use actix_web::web::Json;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::adapters::repositories::environment_repository::environment_repository_factory;
use crate::adapters::repositories::feature_flags_repository::feature_flags_repository_factory;
use crate::AppState;
use crate::domain::models::{Environment, FeatureFlag};
use crate::resources::CustomError;
use crate::services::{environment_handlers, feature_flag_handlers, ServiceError};


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

async fn set_flag(
    data: web::Data<AppState>,
    id: web::Path<String>,
    body: Json<FeatureFlag>
) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = environment_repository_factory(db).await;

    let new_flag = FeatureFlag {
        id: None,
        name: body.name.clone(),
        label: body.label.clone(),
        enabled: body.enabled,
        rules: body.rules.clone(),
    };

    let env_id = id.into_inner();
    match environment_handlers::set_flag(&repo, &env_id, &new_flag).await {
        Ok(env) => Ok(HttpResponse::Accepted().json(Json(env))),
        Err(_) => Err(CustomError::NotFound)
    }
}

async fn remove_flag(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = environment_repository_factory(db).await;
    let (env_id, flag_name) = path.into_inner();

    match environment_handlers::remove_flag(&repo, &env_id, &flag_name).await {
        Ok(env) => Ok(HttpResponse::Accepted().json(Json(env))),
        Err(_) => Err(CustomError::NotFound)
    }

}

pub fn create_scope() -> Scope {
    web::scope("/environments")
        .route("", web::get().to(find))
        .route("/{id}", web::get().to(get))
        .route("", web::post().to(create))
        .route("/{id}", web::delete().to(delete))
        .route("/{id}/flags", web::put().to(set_flag))
        .route("/{id}/flags/{name}", web::delete().to(remove_flag))
}


#[cfg(test)]
mod tests {
    use std::process::id;
    use actix_web::{App, test};
    use actix_web::http::StatusCode;
    use mongodb::bson::doc;
    use serde_json::json;
    use crate::adapters::repositories::environment_repository::environment_repository_factory;
    use crate::adapters::repositories::feature_flags_repository::feature_flags_repository_factory;
    use crate::AppState;
    use crate::database::init_db;
    use crate::domain::models::Environment;
    use crate::resources::feature_flags_api;
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
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_environment_integration() {
        let db = init_db().await.unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    app_name: String::from("Feature Flags"),
                    db: db.clone(),
                }))
                .service(create_scope()),
        )
        .await;
        // Create a new environment
        let env = Environment::new("dev_integration_test");
        let req = test::TestRequest::post()
            .uri("/environments")
            .set_json(Json(env)).to_request();
        let resp: Environment = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "dev_integration_test");

        // Get env by id
        let req = test::TestRequest::get()
            .uri(&format!("/environments/{}", resp.id.unwrap().to_string()))
            .to_request();
        let resp: Environment = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "dev_integration_test");

        // Delete env
        let req = test::TestRequest::delete()
            .uri(&format!("/environments/{}", resp.id.unwrap()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[actix_web::test]
    async fn test_env_manage_flags() {
        let db = init_db().await.unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    app_name: String::from("Feature Flags"),
                    db: db.clone(),
                }))
                .service(create_scope())
                .service(feature_flags_api::create_scope()),
        )
        .await;

        // Create flag
        let flag = FeatureFlag::new("flag_to_be_added", "Sample Flag", true, vec![]);
        let req = test::TestRequest::post()
            .uri("/feature_flags")
            .set_json(Json(flag.clone()))
            .to_request();
        let resp: FeatureFlag = test::call_and_read_body_json(&app, req).await;
        let flag_id = resp.id.unwrap().to_string();

        // Create env
        let env = Environment::new("test_env_integration");
        let req = test::TestRequest::post()
            .uri("/environments")
            .set_json(Json(env))
            .to_request();
        let resp: Environment = test::call_and_read_body_json(&app, req).await;

        let env_id = resp.id.unwrap().to_string();
        // Add flag to env
        let req = test::TestRequest::put()
            .uri(&format!("/environments/{}/flags", env_id))
            .set_json(Json(flag))
            .to_request();
        let resp: Environment = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.flags.len(), 1);

        // Remove flag from env
        let req = test::TestRequest::delete()
            .uri(&format!("/environments/{}/flags/{}", &env_id, "flag_to_be_added"))
            .to_request();
        let resp: Environment = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.flags.len(), 0);

        // Delete env
        let req = test::TestRequest::delete()
            .uri(&format!("/environments/{}", &env_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Delete flag
        let req = test::TestRequest::delete()
            .uri(&format!("/feature_flags/{}", &flag_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
    }

}