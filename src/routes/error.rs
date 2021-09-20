use redact_crypto::CryptoError;
use warp::reject::Reject;
use x509_parser::{error::X509Error, nom};

#[derive(Debug)]
pub struct CryptoErrorRejection(pub CryptoError);
impl Reject for CryptoErrorRejection {}

#[derive(Debug)]
pub struct BadRequestRejection;
impl Reject for BadRequestRejection {}

#[derive(Debug)]
pub struct X509ErrorRejection(pub nom::Err<X509Error>);
impl Reject for X509ErrorRejection {}
