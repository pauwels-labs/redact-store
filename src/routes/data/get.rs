use crate::routes::error::{BadRequestRejection, DataStorageErrorRejection};
use redact_data::DataStorer;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

#[derive(Serialize, Deserialize)]
struct GetQueryParams {
    skip: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Serialize)]
struct GetCollectionResponse<T: Serialize> {
    results: Vec<T>,
}

pub fn get<T: DataStorer>(
    data_storer: T,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!(String)
        .map(|data_path| data_path)
        .and(
            warp::query::<GetQueryParams>().and_then(|query: GetQueryParams| async move {
                if let Some(page_size) = query.page_size {
                    if page_size > 100 {
                        Err(warp::reject::custom(BadRequestRejection))
                    } else {
                        Ok(query)
                    }
                } else {
                    Ok(query)
                }
            }),
        )
        .and(warp::any().map(move || data_storer.clone()))
        .and_then(
            move |data_path: String, query: GetQueryParams, data_storer: T| async move {
                if let Some(skip) = query.skip {
                    let page_size = if let Some(page_size) = query.page_size {
                        page_size
                    } else {
                        10
                    };

                    let data_coll = data_storer
                        .get_collection(&data_path, skip, page_size)
                        .await
                        .map_err(|e| warp::reject::custom(DataStorageErrorRejection(e)))?;
                    Ok::<_, Rejection>(warp::reply::json(&GetCollectionResponse {
                        results: data_coll.0,
                    }))
                } else {
                    let data = data_storer
                        .get(&data_path)
                        .await
                        .map_err(|e| warp::reject::custom(DataStorageErrorRejection(e)))?;
                    Ok::<_, Rejection>(warp::reply::json(&data))
                }
            },
        )
}
