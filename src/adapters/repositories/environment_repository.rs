use crate::adapters::repositories::{
    init_collection, BaseRepository, ErrorKind, Repository, RepositoryError,
};
use crate::domain::models::Environment;
use async_trait::async_trait;
use mongodb::bson::{doc, to_document};
use mongodb::Collection;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub async fn environment_repository_factory() -> EnvironmentRepository<Environment> {
    EnvironmentRepository::<Environment>::new("environments").await
}

pub struct EnvironmentRepository<T> {
    pub(crate) collection: Collection<T>,
}

impl<T> EnvironmentRepository<T> {
    pub async fn new(collection_name: &str) -> EnvironmentRepository<T> {
        let collection = init_collection::<T>(collection_name).await;
        Self { collection }
    }
}

#[async_trait]
impl<T> BaseRepository<T> for EnvironmentRepository<T>
where
    T: Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    fn collection(&self) -> &Collection<T> {
        &self.collection
    }

    async fn create(&self, entity: &T) -> Result<String, RepositoryError> {
        let name = to_document(entity).unwrap().get("name").unwrap().to_owned();
        let existing = self
            .find(doc! {
                "name": name.clone()
            })
            .await;
        if let Ok(items) = existing {
            if !items.is_empty() {
                return Err(RepositoryError::new(
                    ErrorKind::AlreadyExists,
                    format!("Environment with name {} already exists", &name),
                ));
            }
        }
        let res = self
            .collection()
            .insert_one(entity, None)
            .await
            .expect("Error creating entity")
            .inserted_id
            .as_object_id()
            .expect("Failed parsing object id");
        Ok(res.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::repositories::BaseRepository;
    use crate::domain::models::{FeatureFlag, Operator, Rule};
    use std::collections::HashSet;

    #[actix_web::test]
    async fn test_environment_repository() {
        let environment = environment_repository_factory().await;
    }

    #[actix_web::test]
    async fn test_create_environment() {
        let repo = environment_repository_factory().await;
        let environment = Environment::new("development");
        let res = repo.create(&environment).await;
        if let Ok(inserted_id) = res {
            let r = repo.delete(&inserted_id).await;
            assert!(r.is_ok());
        }
    }

    #[actix_web::test]
    async fn test_cannot_create_duplicated_environment() {
        let repo = environment_repository_factory().await;
        let environment = Environment::new("existing_env");
        if let Ok(inserted_id) = repo.create(&environment).await {
            let res = repo.create(&environment).await;
            assert!(res.is_err());
            let err = res.unwrap_err();
            assert_eq!(err.kind, ErrorKind::AlreadyExists);

            let res = repo.delete(&inserted_id).await;
            assert!(res.is_ok())
        }
    }

    #[actix_web::test]
    async fn test_create_environment_with_flags() {
        let repo = environment_repository_factory().await;
        let mut environment = Environment::new("development");

        let flag = FeatureFlag::new("sample_flag", "Sample Flag");
        environment.add_flag(&flag);
        let res = repo.create(&environment).await;
        if let Ok(inserted_id) = res {
            let res = repo.delete(&inserted_id).await;
            assert!(res.is_ok());
        }
    }

    #[actix_web::test]
    async fn test_update_environment() {
        let repo = environment_repository_factory().await;
        let mut environment = Environment::new("development");

        let flag = FeatureFlag::new("sample_flag", "Sample Flag");
        environment.set_flags(HashSet::from([flag]));
        match repo.create(&environment).await {
            Ok(inserted_id) => {
                environment.set_flags(HashSet::new());
                let res = repo.update(&inserted_id, &environment).await;
                assert!(res.is_ok());
                match res {
                    Ok(_) => {
                        let res = repo.delete(&inserted_id).await;
                        assert!(res.is_ok());
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    }
}