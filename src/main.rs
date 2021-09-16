mod routes;

use redact_config::Configurator;
use redact_crypto::storage::gcs::GoogleCloudStorer;
use redact_crypto::storage::NonIndexedTypeStorer::GoogleCloud;
use redact_crypto::storage::TypeStorer::NonIndexedTypeStorer;
use redact_crypto::{MongoStorer, TypeStorer};
use serde::Serialize;
use std::sync::Arc;
use warp::Filter;

#[derive(Serialize)]
struct Healthz {}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Extract config with a REDACT_ env var prefix
    let config = redact_config::new("REDACT").unwrap();

    // Determine port to listen on
    let port = match config.get_int("server.port") {
        Ok(port) => {
            if !(1..=65535).contains(&port) {
                println!(
                    "listen port value '{}' is not between 1 and 65535, defaulting to 8080",
                    port
                );
                8080
            } else {
                port as u16
            }
        }
        Err(e) => {
            match e {
                // Suppress debug logging if server.port was simply not set
                redact_config::ConfigError::NotFound(_) => {
                    println!("listen port not set in config, defaulting to 8080")
                }
                _ => println!("{}", e),
            }
            8080
        }
    };

    // Extract handle to the database
    let db_url = config.get_str("db.url").unwrap();
    let db_name = config.get_str("db.name").unwrap();
    let mongo_storer = Arc::new(MongoStorer::new(&db_url, &db_name));

    let storage_bucket_name = config.get_str("google.storage.bucket.name").unwrap();
    let google_storer = Arc::new(TypeStorer::NonIndexedTypeStorer(GoogleCloud(
        GoogleCloudStorer::new(storage_bucket_name),
    )));

    // Build out routes
    let health_get = warp::path!("healthz")
        .and(warp::get())
        .map(|| warp::reply::json(&Healthz {}));
    let get = warp::get().and(routes::get::get(mongo_storer.clone()));
    let post = warp::post().and(routes::post::create(
        mongo_storer.clone(),
        google_storer.clone(),
    ));

    let routes = health_get.or(get).or(post).with(warp::log("routes"));

    let cert_path = config.get_str("tls.server.certificate.path").unwrap();
    let key_path = config.get_str("tls.server.key.path").unwrap();
    let client_ca_path = config.get_str("tls.client.ca.path").unwrap();

    // Start the server
    println!("starting server listening on ::{}", port);
    warp::serve(routes)
        .tls()
        .cert_path(cert_path)
        .key_path(key_path)
        .client_auth_required_path(client_ca_path)
        .run(([0, 0, 0, 0], port))
        .await;
}
