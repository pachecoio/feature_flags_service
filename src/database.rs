use dotenv;
use mongodb::options::ClientOptions;
use mongodb::{error::Error, Client, Database};
use serde::{Deserialize, Serialize};
use std::env;

pub async fn init_db() -> Result<Database, Error> {
    dotenv::dotenv().ok();
    let uri = env::var("MONGODB_URI").expect("MONGODB_URI not set");
    let db_name = env::var("DATABASE_NAME").expect("DATABASE_NAME not set");

    let client = connection_client(&uri).await?;
    Ok(client.database(&db_name))
}

pub async fn connection_client(uri: &str) -> Result<Client, Error> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(uri).await?;

    // Manually set an option.
    client_options.app_name = Some("Feature Flags".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    // List the names of the databases in that deployment.
    // for db_name in client.list_database_names(None, None).await? {
    //     println!("{}", db_name);
    // }
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_get_db() {
        let db = init_db().await;
        assert!(db.is_ok());
    }

    #[actix_web::test]
    async fn test_add_book() {
        let db = init_db().await.unwrap();
        #[derive(Debug, Serialize, Deserialize)]
        struct Book {
            title: String,
            author: String,
        }

        // Get a handle to a collection of `Book`.
        let book_collection = db.collection::<Book>("books");

        let books = vec![
            Book {
                title: "The Grapes of Wrath".to_string(),
                author: "John Steinbeck".to_string(),
            },
            Book {
                title: "To Kill a Mockingbird".to_string(),
                author: "Harper Lee".to_string(),
            },
        ];

        // Insert the books into "mydb.books" collection, no manual conversion to BSON necessary.
        book_collection.insert_many(books, None).await.unwrap();
    }
}
