use futures::StreamExt;
use crate::adapters::repositories::environment_repository::{EnvironmentRepository};
use crate::adapters::repositories::BaseRepository;
use crate::domain::models::{Environment, FeatureFlag};
use crate::services::ServiceError;
use mongodb::bson::{doc, to_document};
use mongodb::bson::oid::ObjectId;
use serde::{Serialize};

pub async fn find(
    repo: &EnvironmentRepository<Environment>,
    filters: impl Into<Option<Filters>> + Send,
) -> Result<Vec<Environment>, ServiceError> {
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

pub async fn create(
    repo: &EnvironmentRepository<Environment>,
    name: &str,
) -> Result<String, ServiceError> {
    let env = Environment::new(name);
    match repo.create(&env).await {
        Ok(id) => Ok(id),
        Err(err) => Err(ServiceError {
            message: err.to_string(),
        }),
    }
}

pub async fn get(
    repo: &EnvironmentRepository<Environment>,
    id: &str,
) -> Result<Environment, ServiceError> {
    match repo.get(id).await {
        Ok(item) => Ok(item),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn get_by_name(
    repo: &EnvironmentRepository<Environment>,
    name: &str,
) -> Result<Environment, ServiceError> {
    match repo
        .collection
        .find_one(
            doc! {
                "name": name
            },
            None,
        )
        .await
    {
        Ok(res) => match res {
            Some(env) => Ok(env),
            None => Err(ServiceError {
                message: format!("Environment not found with name {}", name),
            }),
        },
        Err(err) => Err(ServiceError {
            message: err.to_string(),
        }),
    }
}

pub async fn update(
    repo: &EnvironmentRepository<Environment>,
    id: &str,
    env: &Environment,
) -> Result<(), ServiceError> {
    match repo.update(id, env).await {
        Ok(_) => Ok(()),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn delete(
    repo: &EnvironmentRepository<Environment>,
    id: &str,
) -> Result<(), ServiceError> {
    match repo.delete(id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn set_flag(
    repo: &EnvironmentRepository<Environment>,
    id: &str,
    flag: &FeatureFlag
) -> Result<Environment, ServiceError> {
    match get(&repo, id).await {
        Ok(mut env) => {
            env.add_flag(flag);
            env.id = Option::from(ObjectId::parse_str(&id).unwrap());
            match repo.update(id, &env).await {
                Ok(_) => Ok(env),
                Err(e) => Err(ServiceError {
                    message: e.to_string(),
                })
            }
        }
        Err(e) => Err(e)
    }
}

pub async fn remove_flag(
    repo: &EnvironmentRepository<Environment>,
    id: &str,
    flag_name: &str
) -> Result<Environment, ServiceError> {
    match get(&repo, id).await {
        Ok(mut env) => {
            env.remove_flag_by_name(&flag_name);
            env.id = Option::from(ObjectId::parse_str(&id).unwrap());
            match repo.update(id, &env).await {
                Ok(_) => Ok(env),
                Err(e) => Err(ServiceError {
                    message: e.to_string(),
                })
            }
        }
        Err(e) => Err(e)
    }
}

#[derive(Serialize, Debug)]
pub struct Filters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::adapters::repositories::environment_repository::environment_repository_factory;
    use crate::adapters::repositories::feature_flags_repository::feature_flags_repository_factory;
    use super::*;
    use crate::database::init_db;
    use crate::domain::models::FeatureFlag;
    use crate::services::feature_flag_handlers;

    #[actix_web::test]
    async fn test_create() {
        let db = init_db().await.unwrap();
        let repo = environment_repository_factory(&db).await;
        let res = create(&repo, "development").await;
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
        let repo = environment_repository_factory(&db).await;
        let res = create(&repo, "services_test").await;
        assert!(res.is_ok());
        match res {
            Ok(id) => {
                let mut env = Environment::new("services_test");
                let flag = FeatureFlag::new("sample_flag", "Sample Flag", true, vec![]);
                env.add_flag(&flag);
                update(&repo, &id, &env).await.unwrap();
                let item = get(&repo, &id).await.unwrap();
                assert_eq!(item.name, "services_test");
                assert_eq!(item.flags.len(), 1);
                delete(&repo, &id).await.unwrap();
            }
            Err(_) => {}
        }
    }

    #[actix_web::test]
    async fn test_manage_flags() {
        let db = init_db().await.unwrap();
        let repo = environment_repository_factory(&db).await;

        let inserted_id = create(&repo, "services_test_env").await.unwrap();

        let flag_repo = feature_flags_repository_factory(&db).await;
        let inserted_flag_id = feature_flag_handlers::create(
            &flag_repo,
            "flag_to_be_managed",
            "Flag to be managed",
            false,
            &vec![]
        ).await.unwrap();

        let flag = FeatureFlag::new(
            "flag_to_be_managed",
            "Flag to be managed",
            true, vec![]
        );

        let res = set_flag(&repo, &inserted_id, &flag).await;
        assert!(res.is_ok());

        let res = get(&repo, &inserted_id).await.unwrap();
        assert_eq!(res.flags.len(), 1);

        let res = remove_flag(&repo, &inserted_id, "flag_to_be_managed").await;
        assert!(res.is_ok());

        let res = get(&repo, &inserted_id).await.unwrap();
        assert_eq!(res.flags.len(), 0);

        delete(&repo, &inserted_id).await.unwrap();
        flag_repo.collection.delete_one(doc! {"name": "flag_to_be_managed"}, None).await.unwrap();
    }
}
