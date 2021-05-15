use crate::routes::error::StorageErrorRejection;
use crate::storage::keys::KeyStorer;
use redact_crypto::keys::Key;
use serde::Serialize;
use warp::{Filter, Rejection, Reply};

#[derive(Serialize)]
struct CreateResponse {
    success: bool,
    msg: String,
}

pub fn create<T: KeyStorer>(
    key_storer: T,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path::end()
        .and(warp::body::content_length_limit(1024 * 1024 * 250))
        .and(warp::body::json::<Key>())
        .and(warp::any().map(move || key_storer.clone()))
        .and_then(move |contents: Key, key_storer: T| async move {
            let success = key_storer
                .create(contents)
                .await
                .map_err(|e| warp::reject::custom(StorageErrorRejection(e)))?;

            Ok::<_, Rejection>(warp::reply::json(&CreateResponse {
                success,
                msg: "inserted".to_owned(),
            }))
        })
}
