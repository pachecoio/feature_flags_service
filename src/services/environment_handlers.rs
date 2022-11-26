use std::fmt::format;
use mongodb::bson::doc;
use crate::adapters::repositories::environment_repository::environment_repository_factory;
use crate::adapters::repositories::BaseRepository;
use crate::domain::models::Environment;
use crate::services::ServiceError;
use mongodb::error::Error;

pub async fn create(name: &str) -> Result<String, ServiceError> {
    let repo = environment_repository_factory().await;
    let env = Environment::new(name);
    match repo.create(&env).await {
        Ok(id) => Ok(id),
        Err(err) => Err(ServiceError {
            message: err.to_string(),
        }),
    }
}

pub async fn get(id: &str) -> Result<Environment, ServiceError> {
    let repo = environment_repository_factory().await;
    match repo.get(&id).await {
        Ok(item) => Ok(item),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn get_by_name(name: &str) -> Result<Environment, ServiceError> {
    let repo = environment_repository_factory().await;
    match repo.collection.find_one(doc! {
        "name": name
    }, None).await {
        Ok(res) => {
            match res {
                Some(env) => Ok(env),
                None => Err(ServiceError {
                    message: format!("Environment not found with name {}", name),
                })
            }
        },
        Err(err) => Err(ServiceError {
            message: format!("Error getting environment. {}", err.to_string())
        })
    }
}

pub async fn update(id: &str, env: &Environment) -> Result<(), ServiceError> {
    let repo = environment_repository_factory().await;
    match repo.update(&id, env).await {
        Ok(_) => Ok(()),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}

pub async fn delete(id: &str) -> Result<(), ServiceError> {
    let repo = environment_repository_factory().await;
    match repo.delete(id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(ServiceError {
            message: e.to_string(),
        }),
    }
}


#[cfg(test)]
mod tests {
    use crate::domain::models::FeatureFlag;
    use super::*;

    #[actix_web::test]
    async fn test_create() {
        let res = create("development").await;
        assert!(res.is_ok());
        match res {
            Ok(id) => {
                delete(&id).await.unwrap();
            }
            Err(_) => {}
        }
    }

    #[actix_web::test]
    async fn test_update() {
        let res = create("services_test").await;
        assert!(res.is_ok());
        match res {
            Ok(id) => {
                let mut env = Environment::new("services_test");
                let flag = FeatureFlag::new("sample_flag", "Sample Flag");
                env.add_flag(&flag);
                update(&id, &env).await.unwrap();
                let item = get(&id).await.unwrap();
                assert_eq!(item.name, "services_test");
                assert_eq!(item.flags.len(), 1);
                delete(&id).await.unwrap();
            }
            Err(_) => {}
        }
    }
}
