use redact_crypto::StorageError as KeyStorageError;
use redact_data::StorageError as DataStorageError;
use warp::reject::Reject;

#[derive(Debug)]
pub struct KeyStorageErrorRejection(pub KeyStorageError);
impl Reject for KeyStorageErrorRejection {}

#[derive(Debug)]
pub struct DataStorageErrorRejection(pub DataStorageError);
impl Reject for DataStorageErrorRejection {}

#[derive(Debug)]
pub struct BadRequestRejection;
impl Reject for BadRequestRejection {}
