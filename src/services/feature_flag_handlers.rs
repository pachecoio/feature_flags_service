use crate::adapters::repositories::feature_flags_repository::{feature_flags_repository_factory, FeatureFlagRepository};
use crate::adapters::repositories::BaseRepository;
use crate::domain::models::FeatureFlag;
use crate::services::ServiceError;
use mongodb::bson::{doc, to_document};
use serde::Serialize;

pub async fn create(repo: &FeatureFlagRepository<FeatureFlag>, name: &str, label: &str) -> Result<String, ServiceError> {
    let inserted_id = repo.create(&FeatureFlag::new(name, label)).await;
    match inserted_id {
        Ok(id) => Ok(id.clone()),
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

pub async fn get(repo: &FeatureFlagRepository<FeatureFlag>, id: &str) -> Result<FeatureFlag, ServiceError> {
    let res = repo.get(&id).await;
    match res {
        Ok(flag) => Ok(flag),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn update(repo: &FeatureFlagRepository<FeatureFlag>, id: &str, label: &str) -> Result<(), ServiceError> {
    match repo.get(&id).await {
        Ok(mut feature_flag) => {
            if feature_flag.label == label {
                return Ok(());
            }

            feature_flag.label = label.to_string();

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

pub async fn delete(repo: &FeatureFlagRepository<FeatureFlag>, id: &str) -> Result<(), ServiceError> {
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
    use crate::database::init_db;
    use super::*;

    #[actix_web::test]
    async fn test_create() {
        let db = init_db().await.unwrap();
        let repo = feature_flags_repository_factory(&db).await;
        let res = create(&repo, "test", "test").await;
        assert!(res.is_ok());
        match res {
            Ok(id) => {
                delete(&repo, &id).await.unwrap();
            }
            Err(_) => {}
        }
    }

    #[actix_web::test]
    async fn test_update() {
        let db = init_db().await.unwrap();
        let repo = feature_flags_repository_factory(&db).await;
        let res = create(&repo, "test", "test").await;
        assert!(res.is_ok());
        match res {
            Ok(id) => {
                update(&repo, &id, "new_label").await.unwrap();
                let res = get(&repo, &id).await.unwrap();
                assert_eq!(res.label, "new_label");
                delete(&repo, &id).await.unwrap();
            }
            Err(_) => {}
        }
    }
}