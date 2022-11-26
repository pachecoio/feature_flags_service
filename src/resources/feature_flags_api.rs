use crate::domain::models::FeatureFlag;
use actix_web::web::Json;
use actix_web::{web, HttpRequest, Result, Scope};
use serde::{Deserialize, Serialize};

async fn find() -> Result<Json<FeatureFlagList>> {
    let flag = FeatureFlag::new("sample_flag", "Sample Flag");
    Ok(Json(FeatureFlagList { items: vec![flag] }))
}

async fn get(_id: web::Path<String>) -> Result<Json<FeatureFlag>> {
    let flag = FeatureFlag::new("sample_flag", "Sample Flag");
    Ok(Json(flag))
}

async fn create(body: Json<FeatureFlag>) -> Result<Json<FeatureFlag>> {
    Ok(body)
}

fn create_scope() -> Scope {
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
    use actix_web::{
        http::{self, header::ContentType},
        test, App,
    };

    #[actix_web::test]
    async fn test_find() {
        let app = test::init_service(App::new().service(create_scope())).await;
        let req = test::TestRequest::get().uri("/feature_flags").to_request();
        let resp: FeatureFlagList = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.items[0].name, "sample_flag");
        assert_eq!(resp.items[0].label, "Sample Flag");
    }

    #[actix_web::test]
    async fn test_get() {
        let app = test::init_service(App::new().service(create_scope())).await;
        let req = test::TestRequest::get()
            .uri("/feature_flags/123")
            .to_request();
        let resp: FeatureFlag = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "sample_flag");
        assert_eq!(resp.label, "Sample Flag");
        assert_eq!(resp.enabled, false);
    }

    #[actix_web::test]
    async fn test_create() {
        let app = test::init_service(App::new().service(create_scope())).await;
        let flag = FeatureFlag {
            id: None,
            name: "sample_flag".to_string(),
            label: "Sample Flag".to_string(),
            enabled: true,
            rules: vec![Rule {
                parameter: "tenant".to_string(),
                operator: Operator::Is("tenant_1".to_string()),
            }],
        };
        let req = test::TestRequest::post()
            .uri("/feature_flags")
            .set_json(Json(flag))
            .to_request();
        let resp: FeatureFlag = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "sample_flag");
        assert_eq!(resp.rules.len(), 1);
    }
}
