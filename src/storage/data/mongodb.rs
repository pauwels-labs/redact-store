use crate::storage::{error::StorageError, Data, DataCollection, DataStorer};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{bson, options::ClientOptions, options::FindOneOptions, Client, Database};

#[derive(Clone)]
pub struct MongoDataStorer {
    url: String,
    db_name: String,
    client: Client,
    db: Database,
}

impl MongoDataStorer {
    pub async fn new(url: &str, db_name: &str) -> Self {
        let db_client_options = ClientOptions::parse_with_resolver_config(
            url,
            mongodb::options::ResolverConfig::cloudflare(),
        )
        .await
        .unwrap();
        let client = Client::with_options(db_client_options).unwrap();
        let db = client.database(db_name);
        MongoDataStorer {
            url: url.to_owned(),
            db_name: db_name.to_owned(),
            client,
            db,
        }
    }
}

#[async_trait]
impl DataStorer for MongoDataStorer {
    async fn get(&self, path: &str) -> Result<Data, StorageError> {
        let filter_options = FindOneOptions::builder().build();
        let filter = bson::doc! { "path": path };

        match self
            .db
            .collection_with_type::<Data>("data")
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

    async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
    ) -> Result<DataCollection, StorageError> {
        let filter_options = mongodb::options::FindOptions::builder()
            .skip(skip)
            .limit(page_size)
            .build();
        let filter = bson::doc! { "path": path };

        match self
            .db
            .collection_with_type::<Data>("data")
            .find(filter, filter_options)
            .await
        {
            Ok(mut cursor) => {
                let mut data = Vec::new();
                while let Some(item) = cursor.next().await {
                    data.push(item.unwrap());
                }
                Ok(DataCollection { data })
            }
            Err(e) => Err(StorageError::DbError {
                source: Box::new(e),
            }),
        }
    }

    async fn create(&self, data: Data) -> Result<bool, StorageError> {
        let filter_options = mongodb::options::ReplaceOptions::builder()
            .upsert(true)
            .build();
        let filter = bson::doc! { "path": data.path.to_string() };

        match self
            .db
            .collection_with_type::<Data>("data")
            .replace_one(filter, data, filter_options)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => Err(StorageError::DbError {
                source: Box::new(e),
            }),
        }
    }
}
