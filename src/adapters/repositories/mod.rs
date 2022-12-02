pub mod environment_repository;
pub mod feature_flags_repository;
use async_trait::async_trait;
use futures::stream::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, to_document, Document};
use mongodb::error::Error;
use mongodb::{Collection, Database};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Display, format, Formatter};

#[async_trait]
pub trait BaseRepository<T>
where
    T: Serialize + Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    fn collection(&self) -> &Collection<T>;

    async fn create(&self, entity: &T) -> Result<String, RepositoryError> {
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

    async fn get(&self, id: &str) -> Result<T, RepositoryError> {
        let obj_id = match ObjectId::parse_str(id) {
            Ok(entity_id) => entity_id,
            Err(err) => {
                return Err(RepositoryError {
                    message: err.to_string(),
                    kind: ErrorKind::NotFound,
                })
            }
        };
        let filter = doc! {"_id": obj_id};
        match self.collection().find_one(filter, None).await {
            Ok(res) => match res {
                None => Err(RepositoryError {
                    message: "Entity not found".to_string(),
                    kind: ErrorKind::NotFound,
                }),
                Some(item) => Ok(item),
            },
            Err(err) => Err(RepositoryError {
                message: format!("Error getting entity: {}", err.to_string()),
                kind: ErrorKind::NotFound,
            }),
        }
    }

    async fn find(
        &self,
        filter: impl Into<Option<Document>> + Send,
    ) -> Result<Vec<T>, RepositoryError> {
        match self.collection().find(filter, None).await {
            Ok(mut cursors) => {
                let mut res = Vec::<T>::new();
                while let Some(f) = cursors
                    .try_next()
                    .await
                    .expect("Error mapping through cursor")
                {
                    res.push(f)
                }
                Ok(res)
            }
            Err(err) => Err(RepositoryError {
                message: err.to_string(),
                kind: ErrorKind::NotFound,
            }),
        }
    }

    async fn update(&self, id: &str, entity: &T) -> Result<(), Error> {
        let obj_id =
            ObjectId::parse_str(id).expect("Error parsing id as ObjectID, id should be a string");
        let filter = doc! {"_id": obj_id};
        let doc = to_document(entity)?;
        let new_doc = doc! {
            "$set": doc
        };
        self.collection()
            .update_one(filter, new_doc, None)
            .await
            .ok()
            .unwrap_or_else(|| panic!("Document not found with id {}", id));

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        let obj_id =
            ObjectId::parse_str(id).expect("Error parsing id as ObjectID, id should be a string");
        let filter = doc! {"_id": obj_id};
        self.collection()
            .find_one_and_delete(filter, None)
            .await
            .ok()
            .unwrap();
        Ok(())
    }
}

pub struct Repository<T> {
    collection: Collection<T>,
}

impl<T> BaseRepository<T> for Repository<T>
where
    T: Serialize + DeserializeOwned + Unpin + Send + Sync,
{
    fn collection(&self) -> &Collection<T> {
        &self.collection
    }
}

impl<T> Repository<T> {
    pub(crate) async fn new(db: &Database, collection_name: &str) -> Repository<T> {
        let collection = init_collection::<T>(db, collection_name).await;
        Self { collection }
    }
}

async fn init_collection<T>(db: &Database, collection_name: &str) -> Collection<T> {
    let collection: Collection<T> = db.collection(collection_name);
    collection
}

#[derive(Clone, Debug)]
pub struct RepositoryError {
    pub message: String,
    pub kind: ErrorKind,
}

impl RepositoryError {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { message, kind }
    }
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    NotFound,
    AlreadyExists,
}
