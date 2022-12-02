use crate::adapters::repositories::feature_flags_repository::feature_flags_repository_factory;
use crate::domain::models::{FeatureFlag, Rule};
use crate::resources::CustomError;
use crate::services::{feature_flag_handlers};
use crate::AppState;
use actix_web::web::Json;
use actix_web::{web, HttpResponse, Result, Scope};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

async fn find(data: web::Data<AppState>) -> Result<Json<FeatureFlagList>> {
    let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    let res = feature_flag_handlers::find(&repo, None).await;
    let flags = match res {
        Ok(flags) => flags,
        Err(_) => vec![],
    };
    Ok(Json(FeatureFlagList { items: flags }))
}
async fn get(data: web::Data<AppState>, id: web::Path<String>) -> Result<HttpResponse, CustomError> { let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    let flag_id = id.into_inner();
    match feature_flag_handlers::get(&repo, &flag_id).await {
        Ok(mut flag) => {
            let _id = ObjectId::parse_str(flag_id).unwrap();
            flag.id = Some(_id);
            Ok(HttpResponse::Ok().json(Json(flag)))
        }
        Err(_) => Err(CustomError::NotFound),
    }
}

async fn create(
    data: web::Data<AppState>,
    body: Json<FeatureFlag>,
) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    match feature_flag_handlers::create(
        &repo, &body.name, &body.label, body.enabled, &body.rules,
    ).await {
        Ok(id) => {
            let mut flag = FeatureFlag::new(
                &body.name,
                &body.label,
                body.enabled,
                body.rules.to_vec(),
            );
            let flag_id = ObjectId::parse_str(id).expect("");
            flag.id = Some(flag_id);
            Ok(HttpResponse::Created().json(Json(flag)))
        }
        Err(_) => Err(CustomError::Conflict),
    }
}

async fn update(
    data: web::Data<AppState>,
    body: Json<FeatureFlagUpdateSchema>,
    id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    let flag_id = id.into_inner();
    match feature_flag_handlers::update(&repo, &flag_id, &body.label).await {
        Ok(id) => Ok(HttpResponse::Accepted().finish()),
        Err(_) => Err(CustomError::Conflict),
    }
}

async fn delete(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    let flag_id = id.into_inner();
    match feature_flag_handlers::delete(&repo, &flag_id).await {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(_) => Err(CustomError::NotFound),
    }
}

pub fn create_scope() -> Scope {
    web::scope("/feature_flags")
        .route("", web::get().to(find))
        .route("/{id}", web::get().to(get))
        .route("", web::post().to(create))
        .route("/{id}", web::delete().to(delete))
        .route("/{id}", web::put().to(update))
}

#[derive(Serialize, Deserialize)]
struct FeatureFlagList {
    items: Vec<FeatureFlag>,
}

#[derive(Serialize, Deserialize)]
struct FeatureFlagUpdateSchema {
    label: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Operator, Rule};
    use crate::AppState;
    use actix_web::http::StatusCode;
    use actix_web::{
        http::{self, header::ContentType},
        test, App, HttpResponse,
    };
    use mongodb::bson::doc;
    use std::process::id;
    use chrono::Utc;
    use mongodb::bson;
    use crate::database::init_db;

    #[actix_web::test]
    async fn test_feature_flag_resource() {
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
        let flag = FeatureFlag {
            id: None,
            name: "sample_flag_integration_test".to_string(),
            label: "Sample Flag".to_string(),
            enabled: false,
            rules: vec![
                Rule {
                    parameter: "tenant".to_string(),
                    operator: Operator::Is("tenant1".to_string()),
                }
            ],
            created_at: Utc::now(),
            updated_at: Utc::now()
        };

        // Create flag
        let req = test::TestRequest::post()
            .uri("/feature_flags")
            .set_json(Json(flag))
            .to_request();
        let resp: FeatureFlag = test::call_and_read_body_json(&app, req).await;
        let id = resp.id.unwrap().to_string();
        assert_eq!(resp.name, "sample_flag_integration_test");
        assert_eq!(resp.label, "Sample Flag");
        assert_eq!(resp.rules.len(), 1);

        // Get by id
        let req = test::TestRequest::get()
            .uri(&format!("/feature_flags/{}", &id))
            .to_request();
        let resp: FeatureFlag = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "sample_flag_integration_test");

        // Test update
        let update_flag = FeatureFlagUpdateSchema {
            label: "Updated Label".to_string(),
        };
        let req = test::TestRequest::put()
            .uri(&format!("/feature_flags/{}", &id))
            .set_json(Json(update_flag))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        // Delete item
        let req = test::TestRequest::delete()
            .uri(&format!("/feature_flags/{}", &id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT)
    }
}
