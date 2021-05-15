use crate::storage::StorageError;
use warp::reject::Reject;

#[derive(Debug)]
pub struct StorageErrorRejection(pub StorageError);
impl Reject for StorageErrorRejection {}

#[derive(Debug)]
pub struct BadRequestRejection;
impl Reject for BadRequestRejection {}
