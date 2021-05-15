use crate::storage::{error::StorageError, keys::KeyStorer};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{bson, options::ClientOptions, options::FindOneOptions, Client, Database};
use redact_crypto::keys::{Key, KeyCollection};

#[derive(Clone)]
pub struct MongoKeyStorer {
    url: String,
    db_name: String,
    client: Client,
    db: Database,
}

impl MongoKeyStorer {
    pub async fn new(url: &str, db_name: &str) -> Self {
        let db_client_options = ClientOptions::parse_with_resolver_config(
            url,
            mongodb::options::ResolverConfig::cloudflare(),
        )
        .await
        .unwrap();
        let client = Client::with_options(db_client_options).unwrap();
        let db = client.database(db_name);
        MongoKeyStorer {
            url: url.to_owned(),
            db_name: db_name.to_owned(),
            client,
            db,
        }
    }
}

#[async_trait]
impl KeyStorer for MongoKeyStorer {
    async fn get(&self, name: &str) -> Result<Key, StorageError> {
        let filter_options = FindOneOptions::builder().build();
        let filter = bson::doc! { "name": name };

        match self
            .db
            .collection_with_type::<Key>("data")
            .find_one(filter, filter_options)
            .await
        {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(StorageError::NotFound),
            Err(e) => Err(StorageError::DbError {
                source: Box::new(e),
            }),
        }
    }

    async fn list(&self) -> Result<KeyCollection, StorageError> {
        match self
            .db
            .collection_with_type::<Key>("keys")
            .find(None, None)
            .await
        {
            Ok(mut cursor) => {
                let mut results = Vec::new();
                while let Some(item) = cursor.next().await {
                    results.push(item.unwrap());
                }
                Ok(KeyCollection { results })
            }
            Err(e) => Err(StorageError::DbError {
                source: Box::new(e),
            }),
        }
    }

    async fn create(&self, value: Key) -> Result<bool, StorageError> {
        let filter_options = mongodb::options::ReplaceOptions::builder()
            .upsert(true)
            .build();
        let filter = bson::doc! { "name": &value.name };

        match self
            .db
            .collection_with_type::<Key>("keys")
            .replace_one(filter, value, filter_options)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => Err(StorageError::DbError {
                source: Box::new(e),
            }),
        }
    }
}
