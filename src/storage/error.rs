use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;

/// Error type that converts to a warp::Rejection
#[derive(Debug)]
pub enum StorageError {
    /// Represents an error which occurred while loading a session from
    /// the backing session store.
    DbError {
        source: Box<dyn Error + Send + Sync>,
    },

    /// Indicates the requested data was not found
    NotFound,
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            StorageError::DbError { ref source } => Some(source.as_ref()),
            StorageError::NotFound => None,
        }
    }
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            StorageError::DbError { .. } => {
                write!(f, "Db error")
            }
            StorageError::NotFound { .. } => {
                write!(f, "Requested data was not found")
            }
        }
    }
}
