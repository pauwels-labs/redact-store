use crate::routes::error::StorageErrorRejection;
use redact_crypto::{Entry, Storer};
use serde::Serialize;
use warp::{Filter, Rejection, Reply};

#[derive(Serialize)]
struct CreateResponse {
    success: bool,
    msg: String,
}

pub fn create<T: Storer>(
    storer: T,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path::end()
        .and(warp::body::content_length_limit(1024 * 1024 * 250))
        .and(warp::body::json::<Entry>())
        .and(warp::any().map(move || storer.clone()))
        .and_then(move |entry: Entry, storer: T| async move {
            let success = storer
                .create(entry.path, entry.value)
                .await
                .map_err(|e| warp::reject::custom(StorageErrorRejection(e)))?;

            Ok::<_, Rejection>(warp::reply::json(&CreateResponse {
                success,
                msg: "inserted".to_owned(),
            }))
        })
}
