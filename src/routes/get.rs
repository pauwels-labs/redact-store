use crate::routes::error::{BadRequestRejection, StorageErrorRejection};
use redact_crypto::{StorageError, Storer, Type};
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

#[derive(Serialize)]
struct NotFoundResponse {}

pub fn get<T: Storer>(
    storer: T,
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
        .and(warp::any().map(move || storer.clone()))
        .and_then(
            move |data_path: String, query: GetQueryParams, storer: T| async move {
                if let Some(skip) = query.skip {
                    let page_size = if let Some(page_size) = query.page_size {
                        page_size
                    } else {
                        10
                    };

                    match storer.list::<Type>(&data_path, skip, page_size).await {
                        Ok(results) => Ok::<_, Rejection>(warp::reply::with_status(
                            warp::reply::json(&GetCollectionResponse { results }),
                            warp::http::StatusCode::OK,
                        )),
                        Err(e) => {
                            if let StorageError::NotFound = e {
                                Ok::<_, Rejection>(warp::reply::with_status(
                                    warp::reply::json(&NotFoundResponse {}),
                                    warp::http::StatusCode::NOT_FOUND,
                                ))
                            } else {
                                Err(warp::reject::custom(StorageErrorRejection(e)))
                            }
                        }
                    }
                } else {
                    match storer.get::<Type>(&data_path).await {
                        Ok(data) => Ok::<_, Rejection>(warp::reply::with_status(
                            warp::reply::json(&data),
                            warp::http::StatusCode::OK,
                        )),
                        Err(e) => {
                            if let StorageError::NotFound = e {
                                Ok::<_, Rejection>(warp::reply::with_status(
                                    warp::reply::json(&NotFoundResponse {}),
                                    warp::http::StatusCode::NOT_FOUND,
                                ))
                            } else {
                                Err(warp::reject::custom(StorageErrorRejection(e)))
                            }
                        }
                    }
                }
            },
        )
}
