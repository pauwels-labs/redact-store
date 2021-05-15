use crate::routes::error::StorageErrorRejection;
use crate::storage::keys::KeyStorer;
use serde::Serialize;
use warp::{Filter, Rejection, Reply};

pub fn get<T: KeyStorer>(
    key_storer: T,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!(String)
        .map(|key_name| key_name)
        .and(warp::any().map(move || key_storer.clone()))
        .and_then(move |key_name: String, key_storer: T| async move {
            let key = key_storer
                .get(&key_name)
                .await
                .map_err(|e| warp::reject::custom(StorageErrorRejection(e)))?;
            Ok::<_, Rejection>(warp::reply::json(&key))
        })
}

#[derive(Serialize)]
struct ListResponse<T: Serialize> {
    results: Vec<T>,
}

pub fn list<T: KeyStorer>(
    key_storer: T,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path::end()
        .and(warp::any().map(move || key_storer.clone()))
        .and_then(move |key_storer: T| async move {
            let key_list = key_storer
                .list()
                .await
                .map_err(|e| warp::reject::custom(StorageErrorRejection(e)))?;

            Ok::<_, Rejection>(warp::reply::json(&ListResponse {
                results: key_list.results,
            }))
        })
}
