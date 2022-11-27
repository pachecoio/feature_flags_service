use std::future::Future;
use crate::domain::models::FeatureFlag;
use actix_web::web::Json;
use actix_web::{web, HttpRequest, Result, Scope, Resource, error, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::adapters::repositories::feature_flags_repository::feature_flags_repository_factory;
use crate::AppState;
use crate::database::init_db;
use crate::resources::CustomError;
use crate::services::{feature_flag_handlers, ServiceError};
use mongodb::bson::oid::ObjectId;

async fn find(data: web::Data<AppState>) -> Result<Json<FeatureFlagList>> {
    let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    let res = feature_flag_handlers::find(&repo, None).await;
    let flags = match res {
        Ok(flags) => flags,
        Err(_) => vec![]
    };
    Ok(Json(FeatureFlagList { items: flags }))
}

async fn get(data: web::Data<AppState>, id: web::Path<String>) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    let flag_id = id.into_inner();
    match feature_flag_handlers::get(&repo, &flag_id).await {
        Ok(mut flag) => {
            let _id = ObjectId::parse_str(flag_id).unwrap();
            flag.id = Some(_id);
            Ok(HttpResponse::Ok().json(Json(flag)))
        },
        Err(_) => Err(CustomError::NotFound)
    }
}

async fn create(data: web::Data<AppState>, body: Json<FeatureFlag>) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    match feature_flag_handlers::create(&repo, &body.name, &body.label).await {
        Ok(id) => {
            let mut flag = FeatureFlag::new(
                &body.name,
                &body.label
            );
            let flag_id = ObjectId::parse_str(id).expect("");
            flag.id = Some(flag_id);
            Ok(HttpResponse::Created().json(Json(flag)))
        }
        Err(err) => {
            Err(CustomError::Conflict)
        }
    }
}

async fn delete(data: web::Data<AppState>, id: web::Path<String>) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    let flag_id = id.into_inner();
    match feature_flag_handlers::delete(&repo, &flag_id).await {
        Ok(_) => Ok(HttpResponse::Accepted().finish()),
        Err(_) => Err(CustomError::NotFound)
    }
}

pub fn create_scope() -> Scope {
    web::scope("/feature_flags")
        .route("", web::get().to(find))
        .route("/{id}", web::get().to(get))
        .route("", web::post().to(create))
        .route("/{id}", web::delete().to(delete))
}

#[derive(Serialize, Deserialize)]
struct FeatureFlagList {
    items: Vec<FeatureFlag>,
}

#[cfg(test)]
mod tests {
    use std::process::id;
    use super::*;
    use crate::domain::models::{Operator, Rule};
    use actix_web::{http::{self, header::ContentType}, test, App, HttpResponse};
    use actix_web::http::StatusCode;
    use mongodb::bson::doc;
    use crate::AppState;

    #[actix_web::test]
    async fn test_feature_flag_resource() {
        let db = init_db().await.unwrap();
        let repo = feature_flags_repository_factory(&db).await;
        repo.collection.delete_many(doc! {}, None).await.unwrap();
        let app = test::init_service(
            App::new().app_data(web::Data::new(AppState {
                app_name: String::from("Feature Flags"),
                db: db.clone()
            })).service(create_scope())
        ).await;
        let flag = FeatureFlag::new("sample_flag", "Sample Flag");

        let req = test::TestRequest::post()
            .uri("/feature_flags")
            .set_json(Json(flag))
            .to_request();
        let resp: FeatureFlag = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "sample_flag");
        assert_eq!(resp.label, "Sample Flag");

        let req = test::TestRequest::get()
            .uri(&format!("/feature_flags/{}", resp.id.unwrap().to_string()))
            .to_request();
        let resp: FeatureFlag = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "sample_flag");

        let req = test::TestRequest::delete()
            .uri(&format!("/feature_flags/{}", resp.id.unwrap().to_string()))
            .to_request();
        let resp= test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::ACCEPTED)
    }
}
