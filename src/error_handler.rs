use serde::Serialize;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::{Rejection, Reply};
use crate::routes::error::{BadRequestRejection, NotFoundRejection};

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

// This function receives a `Rejection` and tries to return a custom
// value, otherwise simply passes the rejection along.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;
    if err.find::<NotFoundRejection>().is_some() {
        code = StatusCode::NOT_FOUND;
        message = "NOT FOUND";
    } else if err.find::<BadRequestRejection>().is_some() {
        code = StatusCode::BAD_REQUEST;
        message = "BAD REQUEST";
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL SERVER ERROR";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
