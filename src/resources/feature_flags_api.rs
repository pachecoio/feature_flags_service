use crate::domain::models::FeatureFlag;
use actix_web::web::Json;
use actix_web::{web, HttpRequest, Result, Scope, Resource};
use serde::{Deserialize, Serialize};
use crate::adapters::repositories::feature_flags_repository::feature_flags_repository_factory;
use crate::AppState;
use crate::database::init_db;
use crate::services::{feature_flag_handlers, ServiceError};

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

async fn get(_id: web::Path<String>) -> Result<Json<FeatureFlag>> {
    let flag = FeatureFlag::new("sample_flag", "Sample Flag");
    Ok(Json(flag))
}

async fn create(body: Json<FeatureFlag>) -> Result<Json<FeatureFlag>> {
    Ok(body)
}

pub(crate) fn create_scope() -> Scope {
    web::scope("/feature_flags")
        .route("", web::get().to(find))
        .route("/{id}", web::get().to(get))
        .route("", web::post().to(create))
}

#[derive(Serialize, Deserialize)]
struct FeatureFlagList {
    items: Vec<FeatureFlag>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Operator, Rule};
    use actix_web::{http::{self, header::ContentType}, test, App, HttpResponse};
    use crate::AppState;

    #[actix_web::test]
    async fn test_find() {
        let db = init_db().await.unwrap();
        let app = test::init_service(
            App::new().app_data(web::Data::new(AppState {
                app_name: String::from("Feature Flags"),
                db: db.clone()
            })).service(create_scope())
        ).await;
        let req = test::TestRequest::get().uri("/feature_flags").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_create() {
        let db = init_db().await.unwrap();
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
    }
}
