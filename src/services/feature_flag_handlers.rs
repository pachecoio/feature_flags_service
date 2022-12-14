use chrono::Utc;
use mongodb::bson;
use crate::adapters::repositories::feature_flags_repository::{FeatureFlagRepository};
use crate::adapters::repositories::BaseRepository;
use crate::domain::models::{FeatureFlag, Rule};
use crate::services::ServiceError;
use mongodb::bson::{doc, to_document};
use serde::Serialize;

pub async fn create(
    repo: &FeatureFlagRepository<FeatureFlag>,
    name: &str,
    label: &str,
    enabled: bool,
    rules: &Vec<Rule>,
) -> Result<String, ServiceError> {
    let inserted_id = repo.create(
        &FeatureFlag {
            id: None,
            name: name.to_string(),
            label: label.to_string(),
            enabled,
            rules: rules.to_vec(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    ).await;
    match inserted_id {
        Ok(id) => Ok(id),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn find(
    repo: &FeatureFlagRepository<FeatureFlag>,
    filters: impl Into<Option<Filters>> + Send,
) -> Result<Vec<FeatureFlag>, ServiceError> {
    let _filters = match filters.into() {
        None => doc! {},
        Some(f) => to_document(&f).unwrap(),
    };
    let res = repo.find(_filters).await;
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn get(
    repo: &FeatureFlagRepository<FeatureFlag>,
    id: &str,
) -> Result<FeatureFlag, ServiceError> {
    let res = repo.get(id).await;
    match res {
        Ok(flag) => Ok(flag),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn update(
    repo: &FeatureFlagRepository<FeatureFlag>,
    id: &str,
    label: &str,
    enabled: bool,
    rules: Vec<Rule>
) -> Result<(), ServiceError> {
    match repo.get(id).await {
        Ok(mut feature_flag) => {
            feature_flag.label = label.to_string();
            feature_flag.enabled = enabled;
            feature_flag.rules = rules;

            match repo.update(id, &feature_flag).await {
                Ok(_) => Ok(()),
                Err(e) => Err(ServiceError {
                    message: e.to_string(),
                }),
            }
        }
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn delete(
    repo: &FeatureFlagRepository<FeatureFlag>,
    id: &str,
) -> Result<(), ServiceError> {
    match repo.delete(id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

#[derive(Serialize, Debug)]
pub struct Filters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use actix_web::web::Json;
    use mongodb::bson::Bson::DateTime;
    use crate::adapters::repositories::feature_flags_repository::feature_flags_repository_factory;
    use super::*;
    use crate::database::init_db;
    use crate::domain::models::Operator;

    #[actix_web::test]
    async fn test_create() {
        let db = init_db().await.unwrap();
        let repo = feature_flags_repository_factory(&db).await;
        let res = create(
            &repo,
            "feature_flag_handlers_test",
            "Feature Flag handlers test",
            false,
            &vec![
                Rule {
                    parameter: "tenant".to_string(),
                    operator: Operator::Is("tenant1".to_string()),
                }
            ]
        ).await;
        assert!(res.is_ok());
        match res {
            Ok(id) => {
                let res = repo.get(&id).await.unwrap();
                assert_eq!(res.name, "feature_flag_handlers_test");
                assert_eq!(res.rules.len(), 1);
                delete(&repo, &id).await.unwrap();
            }
            Err(_) => {}
        }
    }

    #[actix_web::test]
    async fn test_update() {
        let db = init_db().await.unwrap();
        let repo = feature_flags_repository_factory(&db).await;
        let res = create(
            &repo,
            "feature_flag_handlers_test_update",
            "test",
            false,
            &vec![]
        ).await;
        assert!(res.is_ok());
        match res {
            Ok(id) => {
                update(&repo, &id, "new_label", true, vec![]).await.unwrap();
                let res = get(&repo, &id).await.unwrap();
                assert_eq!(res.label, "new_label");
                delete(&repo, &id).await.unwrap();
            }
            Err(_) => {}
        }
    }
}
