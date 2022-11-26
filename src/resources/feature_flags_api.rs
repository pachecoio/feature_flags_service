use actix_web::{Result, Scope, web};
use actix_web::web::{Json};
use serde::{Serialize, Deserialize};
use crate::domain::models::{FeatureFlag};

async fn find() -> Result<Json<FeatureFlagList>> {
    let flag = FeatureFlag::new(
        "sample_flag",
        "Sample Flag"
    );
    Ok(Json(FeatureFlagList {
        items: vec![flag]
    }))
}

async fn get(_id: web::Path<String>) -> Result<Json<FeatureFlag>> {
    let flag = FeatureFlag::new(
        "sample_flag",
        "Sample Flag"
    );
    Ok(Json(flag))
}

fn create_scope() -> Scope {
    web::scope("/feature_flags")
        .route("", web::get().to(find))
        .route("/{id}", web::get().to(get))
}

#[derive(Serialize, Deserialize)]
struct FeatureFlagList {
    items: Vec<FeatureFlag>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, http::{self, header::ContentType}, test};

    #[actix_web::test]
    async fn test_find() {
        let app= test::init_service(
            App::new().service(create_scope())
        ).await;
        let req = test::TestRequest::get().uri("/feature_flags").to_request();
        let resp: FeatureFlagList = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.items[0].name, "sample_flag");
        assert_eq!(resp.items[0].label, "Sample Flag");
    }

    #[actix_web::test]
    async fn test_get() {
        let app= test::init_service(
            App::new().service(create_scope())
        ).await;
        let req = test::TestRequest::get().uri("/feature_flags/123").to_request();
        let resp: FeatureFlag = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name, "sample_flag");
        assert_eq!(resp.label, "Sample Flag");
    }

}