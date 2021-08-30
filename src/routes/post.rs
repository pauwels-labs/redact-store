use std::sync::Arc;

use crate::routes::error::CryptoErrorRejection;
use redact_crypto::{Entry, Storer, Type};
use serde::Serialize;
use warp::{Filter, Rejection, Reply};

#[derive(Serialize)]
struct CreateResponse {
    success: bool,
    msg: String,
}

pub fn create<T: Storer>(
    storer: Arc<T>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path::end()
        .and(warp::body::content_length_limit(1024 * 1024 * 250))
        .and(warp::body::json::<Entry<Type>>())
        .and(warp::any().map(move || storer.clone()))
        .and_then(move |entry: Entry<Type>, storer: Arc<T>| async move {
            storer
                .create(entry)
                .await
                .map_err(|e| warp::reject::custom(CryptoErrorRejection(e)))?;

            Ok::<_, Rejection>(warp::reply::json(&CreateResponse {
                success: true,
                msg: "inserted".to_owned(),
            }))
        })
}
