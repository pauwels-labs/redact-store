use std::sync::Arc;
use crate::routes::error::CryptoErrorRejection;
use redact_crypto::{Entry, Storer, Type, TypeBuilder, DataBuilder, State, TypeStorer, Data};
use serde::Serialize;
use warp::{Filter, Rejection, Reply};

#[derive(Serialize)]
struct CreateResponse {
    success: bool,
    msg: String,
}

pub fn create<T: Storer>(
    storer: Arc<T>,
    blob_storer: Arc<TypeStorer>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path::end()
        .and(warp::body::content_length_limit(1024 * 1024 * 250))
        .and(warp::body::json::<Entry<Type>>())
        .and(warp::any().map(move || storer.clone()))
        .and(warp::any().map(move || blob_storer.clone()))
        .and_then(move |entry: Entry<Type>, storer: Arc<T>, blob_storer: Arc<TypeStorer>| async move {
            match entry.builder {
                TypeBuilder::Data(d) => {
                    match d {
                        DataBuilder::Binary(_) => {
                            let ref_entry: Entry<Data> = Entry::new(entry.path.clone(), entry.builder, State::Referenced {
                                path: entry.path.clone(),
                                storer: (*blob_storer).clone()
                            });

                            // TODO: orchestration
                            blob_storer
                                .create(entry)
                                .await
                                .map_err(|e| warp::reject::custom(CryptoErrorRejection(e)))?;

                            storer
                                .create(ref_entry)
                                .await
                                .map_err(|e| warp::reject::custom(CryptoErrorRejection(e)))?;
                        }
                        _ => {
                            storer
                                .create(entry)
                                .await
                                .map_err(|e| warp::reject::custom(CryptoErrorRejection(e)))?;
                        }
                    }
                },
                _ => {
                    storer
                        .create(entry)
                        .await
                        .map_err(|e| warp::reject::custom(CryptoErrorRejection(e)))?;
                }
            }

            Ok::<_, Rejection>(warp::reply::json(&CreateResponse {
                success: true,
                msg: "inserted".to_owned(),
            }))
        })
}
