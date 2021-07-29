use redact_crypto::CryptoError;
use warp::reject::Reject;

#[derive(Debug)]
pub struct CryptoErrorRejection(pub CryptoError);
impl Reject for CryptoErrorRejection {}

#[derive(Debug)]
pub struct BadRequestRejection;
impl Reject for BadRequestRejection {}
