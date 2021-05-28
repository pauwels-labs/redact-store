use crate::routes::error::DataStorageErrorRejection;
use redact_data::{Data, DataStorer};
use serde::Serialize;
use warp::{Filter, Rejection, Reply};

#[derive(Serialize)]
struct CreateResponse {
    success: bool,
    msg: String,
}

pub fn create<T: DataStorer>(
    data_storer: T,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path::end()
        .and(warp::body::content_length_limit(1024 * 1024 * 250))
        .and(warp::body::json::<Data>())
        .and(warp::any().map(move || data_storer.clone()))
        .and_then(move |data: Data, data_storer: T| async move {
            let success = data_storer
                .create(data)
                .await
                .map_err(|e| warp::reject::custom(DataStorageErrorRejection(e)))?;

            Ok::<_, Rejection>(warp::reply::json(&CreateResponse {
                success,
                msg: "inserted".to_owned(),
            }))
        })
}
