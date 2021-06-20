use crate::routes::error::{BadRequestRejection, DataStorageErrorRejection};
use redact_data::DataStorer;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

pub fn get<T: DataStorer>(
    data_storer: T,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!(String)
        .map(|data_path| data_path)
        .and(warp::any().map(move || data_storer.clone()))
        .and_then(
            move |data_path: String, data_storer: T| async move {
                let data = data_storer
                    .get(&data_path)
                    .await
                    .map_err(|e| warp::reject::custom(DataStorageErrorRejection(e)))?;
                Ok::<_, Rejection>(warp::reply::json(&data))
            },
        )
}
