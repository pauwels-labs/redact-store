use crate::routes::error::{
    BadRequestRejection, CryptoErrorRejection, NotFoundRejection, X509ErrorRejection,
};
use redact_crypto::{CryptoError, IndexedStorer, Type};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_rustls::rustls::Certificate;
use warp::{Filter, Rejection, Reply};

#[derive(Serialize, Deserialize)]
struct GetQueryParams {
    skip: Option<u64>,
    page_size: Option<i64>,
}

#[derive(Serialize)]
struct GetCollectionResponse<T: Serialize> {
    results: Vec<T>,
}

pub fn get<T: IndexedStorer>(
    storer: Arc<T>,
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
        .and(warp::ext::get::<Certificate>())
        .and(warp::any().map(move || storer.clone()))
        .and_then(
            move |data_path: String,
            query: GetQueryParams,
            client_cert: Certificate,
            storer: Arc<T>| async move {
                let x509_result = x509_parser::parse_x509_certificate(&client_cert.0);
                let _ = match x509_result {
                    Ok((_, cert)) => Ok(cert),
                    Err(e) => Err(warp::reject::custom(X509ErrorRejection(e))),
                }?;

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
                            if let CryptoError::NotFound { .. } = e {
                                Err(warp::reject::custom(NotFoundRejection))
                            } else {
                                log::error!("An error occurred while retrieving the entries list at path {}: {}", data_path, e);
                                Err(warp::reject::custom(CryptoErrorRejection(e)))
                            }
                        }
                    }
                } else {
                    match storer.get::<Type>(&data_path).await {
                        Ok(data) => {
                            let dereferenced_data = data.dereference().await;

                            match dereferenced_data {
                                Ok(data) => Ok::<_, Rejection>(warp::reply::with_status(
                                    warp::reply::json(&data),
                                    warp::http::StatusCode::OK,
                                )),
                                Err(e) => {
                                    if let CryptoError::NotFound { .. } = e {
                                        Err(warp::reject::custom(NotFoundRejection))
                                    } else {
                                        log::error!("An error occurred while dereferencing the entry at path {}: {}", data_path, e);
                                        Err(warp::reject::custom(CryptoErrorRejection(e)))
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            if let CryptoError::NotFound { .. } = e {
                                Err(warp::reject::custom(NotFoundRejection))
                            } else {
                                log::error!("An error occurred while retrieving the entry at path {}: {}", data_path, e);
                                Err(warp::reject::custom(CryptoErrorRejection(e)))
                            }
                        }
                    }
                }
            },
        )
}
