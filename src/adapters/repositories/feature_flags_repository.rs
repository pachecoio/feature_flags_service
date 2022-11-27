use crate::adapters::repositories::{
    init_collection, BaseRepository, ErrorKind, RepositoryError,
};
use crate::domain::models::FeatureFlag;
use mongodb::bson::{doc, to_document};
use mongodb::{Collection, Database};
use serde::de::DeserializeOwned;
use serde::Serialize;
use async_trait::async_trait;

pub async fn feature_flags_repository_factory(db: &Database) -> FeatureFlagRepository<FeatureFlag> {
    FeatureFlagRepository::<FeatureFlag>::new(db, "feature_flags").await
}

pub struct FeatureFlagRepository<T> {
    pub(crate) collection: Collection<T>,
}

impl<T> FeatureFlagRepository<T> {
    pub async fn new(db: &Database, collection_name: &str) -> FeatureFlagRepository<T> {
        let collection = init_collection::<T>(db, collection_name).await;
        Self { collection }
    }
}

#[async_trait]
impl<T> BaseRepository<T> for FeatureFlagRepository<T>
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
                    format!("Feature flag with name {} already exists", &name),
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
mod test_flag_definition_repository {
    use super::*;
    use crate::adapters::repositories::feature_flags_repository::feature_flags_repository_factory;
    use crate::adapters::repositories::BaseRepository;
    use crate::database::init_db;

    #[actix_web::test]
    async fn repo_create() {
        let db = init_db().await.unwrap();
        let repo = feature_flags_repository_factory(&db).await;
        let entity = FeatureFlag::new("sample_flag", "Sample Flag");
        let res = repo.create(&entity).await;
        assert!(res.is_ok());
        if let Ok(id) = res {
            let item = repo.get(&id).await;
            assert!(item.is_ok());
            let item = item.unwrap();
            assert_eq!(item.name, "sample_flag");

            let deleted = repo.delete(&id).await;
            assert!(deleted.is_ok());
        }
    }

    #[actix_web::test]
    async fn repo_find_all() {
        let db = init_db().await.unwrap();
        let repo = feature_flags_repository_factory(&db).await;
        let res = repo.find(None).await;
        assert!(res.is_ok());
    }

    #[actix_web::test]
    async fn repo_update() {
        let db = init_db().await.unwrap();
        let repo = feature_flags_repository_factory(&db).await;
        let entity = FeatureFlag::new("flag_to_update", "Flag to update");
        let res = repo.create(&entity).await;
        let inserted_id = res.unwrap();

        let entity_to_update = FeatureFlag::new("updated_flag", "Updated flag");
        let res = repo.update(&inserted_id, &entity_to_update).await;
        assert!(res.is_ok());
        let updated_flag = res.unwrap();

        let updated_item = repo.get(&inserted_id).await;
        assert!(updated_item.is_ok());
        let updated_item = updated_item.unwrap();
        assert_eq!(updated_item.name, "updated_flag");
    }

    #[actix_web::test]
    async fn repo_delete() {
        let db = init_db().await.unwrap();
        let repo = feature_flags_repository_factory(&db).await;
        let entity = FeatureFlag::new("flag_to_delete", "Flag to Delete");
        let res = repo.create(&entity).await;
        assert!(res.is_ok());
        let inserted_id = res.unwrap();

        let res = repo.delete(&inserted_id).await;
        assert!(res.is_ok());
    }
}
