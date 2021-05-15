pub mod mongodb;

use crate::storage::error::StorageError;
use async_trait::async_trait;
use serde::{de::Deserializer, Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Serialize, Deserialize, Debug)]
pub struct DataCollection {
    pub data: Vec<Data>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub path: DataPath,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataPath {
    #[serde(deserialize_with = "DataPath::deserialize_path")]
    path: String,
}

impl DataPath {
    pub fn new(path: &str) -> Self {
        let path = Self::validate_path(path);
        Self { path }
    }

    fn deserialize_path<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::validate_path(&String::deserialize(deserializer)?))
    }

    // Ensures that a data entry path begins and ends with a period ('.')
    // Empty strings will return as "."
    // Strings of length 1 where the only char is a period will return as "."
    // All other strings will have periods added to the beginning or end if needed.
    // For now, string containing multiple periods in a row, or composed only of
    // multiple periods, will be accepted and returned as given, with the same
    // behavior as any other standard string of len > 1.
    // This function is implemented as a boolean circuit to avoid iterating through
    // the same string numerous times.
    fn validate_path(path: &str) -> String {
        // Short circuit if path is empty
        if path.is_empty() {
            return ".".to_owned();
        }

        // Collect the first and last characters of the path
        let mut path_chars = path.chars();
        let first_char = path_chars.next();
        let last_char = path_chars.last();

        // Match on the results of char extraction
        match (first_char, last_char) {
            // String length >= 2
            (Some(fc), Some(lc)) => {
                if fc != '.' && lc != '.' {
                    format!(".{}.", path)
                } else if fc == '.' && lc == '.' {
                    path.to_owned()
                } else if fc != '.' {
                    format!(".{}", path)
                } else {
                    format!("{}.", path)
                }
            }
            // String length == 1
            (Some(fc), None) => {
                if fc == '.' {
                    path.to_owned()
                } else {
                    format!(".{}.", path)
                }
            }
            // Impossible case: string length == 0, should never be here because
            // of the short-circuit implemented at the beginning of the function
            (None, None) => panic!(
                "this is an impossible situation; if you have gotten here, \\
	     a short-circuit earlier in the function has failed to function as \\
	     intended"
            ),
            // Impossible case: if this happens we should panic because something is
            // fundamentally wrong with the computing environment and someone should
            // know about it.
            // If the last char is != None, then it MUST BE that the
            // first char is != None, as the last char is collected after the
            // iterator has ticked over one spot to account for the first char,
            // therefore if the iterator finds something in the last() call, then
            // it must be after having collected something from the nth(0) call.
            (None, Some(_)) => panic!(
                "this is an impossible situation; if you have gotten here, \\
	     something has happened that should never happen according to the \\
	     laws of computing and/or the rust compiler. if you have gotten here, \\
	     some major memory or computing trickery has occurred and you should \\
	     be concerned for the integrity of your computing base"
            ),
        }
    }
}

impl Display for DataPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl<'a> From<&'a str> for DataPath {
    fn from(path: &'a str) -> Self {
        Self::new(path)
    }
}

#[async_trait]
pub trait DataStorer: Clone + Send + Sync {
    async fn get(&self, path: &str) -> Result<Data, StorageError>;
    async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
    ) -> Result<DataCollection, StorageError>;
    async fn create(&self, data: Data) -> Result<bool, StorageError>;
}
