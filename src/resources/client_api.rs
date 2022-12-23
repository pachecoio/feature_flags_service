use std::sync::Mutex;
use actix_web::{HttpResponse, Scope, web};
use actix_web::web::{Data, Json};
use mongodb::bson::doc;
use serde_json::{Map, Value};
use crate::adapters::repositories::{BaseRepository, RepositoryError};
use crate::adapters::repositories::feature_flags_repository::feature_flags_repository_factory;
use crate::AppState;
use crate::domain::models::FeatureFlag;
use crate::resources::CustomError;
use serde::{Serialize, Deserialize};
use crate::adapters::repositories::environment_repository::environment_repository_factory;
use crate::services::environment_handlers;

async fn get_flags_from_context(
    data: web::Data<Mutex<AppState>>,
    body: Json<FeatureFlagsContextSchema>,
) -> Result<HttpResponse, CustomError> {

    match get_all_flags(&data).await {
        Ok(all_flags) => {
            let mut valid_flags = Map::new();
            for flag in all_flags {
                valid_flags.insert(
                    String::from(&flag.name),
                    Value::Bool(flag.is_context_valid(&body.context))
                );
            }
            Ok(HttpResponse::Ok().json(Json(valid_flags)))
        }
        Err(err) => Err(CustomError::ApplicationError)
    }
}

async fn get_all_flags(data: &Data<Mutex<AppState>>) -> Result<Vec<FeatureFlag>, RepositoryError> {
    let mut app_data = data.lock().unwrap();
    if app_data.flags.is_empty() {
        let db = &app_data.db;
        let repo = feature_flags_repository_factory(db).await;
        return match repo.find(doc! {"enabled": true}).await {
            Ok(all_flags) => {
                app_data.flags = all_flags;
                Ok(app_data.flags.clone())
                // Ok(all_flags)
            }
            Err(err) => Err(err)
        }
    }
    Ok(app_data.flags.clone())
}

async fn get_environment_flags_from_context(
    data: web::Data<AppState>,
    body: Json<FeatureFlagsContextSchema>,
    environment_name: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let db = &data.db;
    let repo = feature_flags_repository_factory(db).await;
    let env_repo = environment_repository_factory(db).await;
    let name = environment_name.into_inner();

    match environment_handlers::get_by_name(&env_repo, &name).await {
        Ok(env) => {
            match repo.find(doc! {"enabled": true}).await {
                Ok(all_flags) => {
                    let mut valid_flags = env.get_flags_from_context(&body.context);
                    for flag in all_flags {
                        if !valid_flags.contains_key(&flag.name) {
                            valid_flags.insert(
                                String::from(&flag.name),
                                Value::Bool(flag.is_context_valid(&body.context))
                            );
                        }
                    }
                    Ok(HttpResponse::Ok().json(Json(valid_flags)))
                }
                Err(err) => Err(CustomError::ApplicationError)
            }
        },
        Err(_) => return Err(CustomError::NotFound)
    }
}

async fn flags_from_context(
    data: web::Data<AppState>,
    body: Json<FeatureFlagsContextSchema>,
) -> Result<HttpResponse, CustomError> {
    Ok(HttpResponse::Ok().json(Json(Map::new())))
}


#[derive(Serialize, Deserialize)]
struct FeatureFlagsContextSchema {
    context: Map<String, Value>,
}

pub fn create_scope() -> Scope {
    web::scope("/flags")
        .route("", web::post().to(
            get_flags_from_context
        ))
        .route("/{environment_name}", web::post().to(
            get_environment_flags_from_context
        ))
}

#[cfg(test)]
mod tests {
    use actix_web::{App, test};
    use chrono::Utc;
    use crate::database::init_db;
    use crate::domain::models::{Environment, Operator, Rule};
    use crate::resources::{environments_api, feature_flags_api};
    use crate::resources::feature_flags_api::FeatureFlagCreateSchema;
    use super::*;

    #[actix_web::test]
    async fn test_get_client_flags() {
        let db = init_db().await.unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    app_name: String::from("Feature Flags"),
                    db: db.clone(),
                    flags: Vec::new(),
                }))
                .service(feature_flags_api::create_scope())
                .service(create_scope()),
        )
        .await;
        let flag_1 = FeatureFlagCreateSchema {
            name: "flag_1".to_string(),
            label: "Flag 1".to_string(),
            enabled: true,
            rules: vec![
                Rule {
                    parameter: "tenant".to_string(),
                    operator: Operator::Is("tenant1".to_string()),
                }
            ],
        };
        let flag_2 = FeatureFlagCreateSchema {
            name: "flag_2".to_string(),
            label: "Flag 2".to_string(),
            enabled: true,
            rules: vec![
                Rule {
                    parameter: "user".to_string(),
                    operator: Operator::IsOneOf(vec![
                        "user_1".to_string(),
                        "user_2".to_string()
                    ]),
                }
            ],
        };

        let req = test::TestRequest::post()
            .uri("/admin/feature_flags")
            .set_json(Json(flag_1))
            .to_request();
        let resp_1: FeatureFlag = test::call_and_read_body_json(&app, req).await;

        let req = test::TestRequest::post()
            .uri("/admin/feature_flags")
            .set_json(Json(flag_2))
            .to_request();
        let resp_2: FeatureFlag = test::call_and_read_body_json(&app, req).await;

        let id_1 = resp_1.id.unwrap().to_string();
        let id_2 = resp_2.id.unwrap().to_string();

        // Get flags from context
        let mut context = Map::new();
        context.insert("tenant".to_string(), Value::String("tenant1".to_string()));
        let req = test::TestRequest::post()
            .uri("/flags")
            .set_json(Json(FeatureFlagsContextSchema {
                context
            }))
            .to_request();
        let resp: Map<String, Value> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.get("flag_1").unwrap(), &Value::Bool(true));
        assert_eq!(resp.get("flag_2").unwrap(), &Value::Bool(false));

        // Delete item
        let req = test::TestRequest::delete()
            .uri(&format!("/admin/feature_flags/{}", &id_1))
            .to_request();
        let resp = test::call_service(&app, req).await;

         // Delete item
        let req = test::TestRequest::delete()
            .uri(&format!("/admin/feature_flags/{}", &id_2))
            .to_request();
        let resp = test::call_service(&app, req).await;
    }

    #[actix_web::test]
    async fn test_get_environment_flags() {
        let db = init_db().await.unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    app_name: String::from("Feature Flags"),
                    db: db.clone(),
                    flags: Vec::new(),
                }))
                .service(feature_flags_api::create_scope())
                .service(environments_api::create_scope())
                .service(create_scope()),
        ).await;
        let flag_1 = FeatureFlagCreateSchema {
            name: "flag_1".to_string(),
            label: "Flag 1".to_string(),
            enabled: true,
            rules: vec![
                Rule {
                    parameter: "tenant".to_string(),
                    operator: Operator::Is("tenant1".to_string()),
                }
            ],
        };
        let flag_2 = FeatureFlagCreateSchema {
            name: "flag_2".to_string(),
            label: "Flag 2".to_string(),
            enabled: true,
            rules: vec![
                Rule {
                    parameter: "user".to_string(),
                    operator: Operator::IsOneOf(vec![
                        "user_1".to_string(),
                        "user_2".to_string()
                    ]),
                }
            ],
        };

        let req = test::TestRequest::post()
            .uri("/admin/feature_flags")
            .set_json(Json(FeatureFlag {
                id: None,
                name: "flag_1".to_string(),
                label: "Flag 1".to_string(),
                enabled: true,
                rules: vec![
                    Rule {
                        parameter: "tenant".to_string(),
                        operator: Operator::Is("tenant1".to_string()),
                    }
                ],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
            .to_request();
        let resp_1: FeatureFlag = test::call_and_read_body_json(&app, req).await;

        let req = test::TestRequest::post()
            .uri("/admin/feature_flags")
            .set_json(Json(flag_2))
            .to_request();
        let resp_2: FeatureFlag = test::call_and_read_body_json(&app, req).await;

        // Create env
        let env = Environment::new("test_env_integration");
        let req = test::TestRequest::post()
            .uri("/admin/environments")
            .set_json(Json(env))
            .to_request();
        let resp: Environment = test::call_and_read_body_json(&app, req).await;
        let env_id = resp.id.unwrap().to_string();

        // Add flag to env
        let req = test::TestRequest::put()
            .uri(&format!("/admin/environments/{}/flags", env_id))
            .set_json(Json(flag_1))
            .to_request();
        let resp: Environment = test::call_and_read_body_json(&app, req).await;

        let id_1 = resp_1.id.unwrap().to_string();
        let id_2 = resp_2.id.unwrap().to_string();

        // Get flags from context
        let mut context = Map::new();
        context.insert("tenant".to_string(), Value::String("tenant1".to_string()));
        let req = test::TestRequest::post()
            .uri("/flags/test_env_integration")
            .set_json(Json(FeatureFlagsContextSchema {
                context
            }))
            .to_request();
        let resp: Map<String, Value> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.get("flag_1").unwrap(), &Value::Bool(true));
        assert_eq!(resp.get("flag_2").unwrap(), &Value::Bool(false));

        // Delete item
        let req = test::TestRequest::delete()
            .uri(&format!("/admin/feature_flags/{}", &id_1))
            .to_request();
        let resp = test::call_service(&app, req).await;

         // Delete item
        let req = test::TestRequest::delete()
            .uri(&format!("/admin/feature_flags/{}", &id_2))
            .to_request();
        let resp = test::call_service(&app, req).await;

         // Delete env
        let req = test::TestRequest::delete()
            .uri(&format!("/admin/environments/{}", &env_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
    }
}