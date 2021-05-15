pub mod mongodb;

use crate::storage::error::StorageError;
use async_trait::async_trait;
use redact_crypto::keys::{Key, KeyCollection};

#[async_trait]
pub trait KeyStorer: Clone + Send + Sync {
    async fn get(&self, name: &str) -> Result<Key, StorageError>;
    async fn list(&self) -> Result<KeyCollection, StorageError>;
    async fn create(&self, value: Key) -> Result<bool, StorageError>;
}
