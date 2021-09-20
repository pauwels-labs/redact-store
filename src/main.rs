mod bootstrap;
mod routes;

use redact_config::Configurator;
use redact_crypto::storage::gcs::GoogleCloudStorer;
use redact_crypto::storage::NonIndexedTypeStorer::GoogleCloud;
use redact_crypto::{MongoStorer, TypeStorer};
use rustls::{internal::pemfile, AllowAnyAuthenticatedClient, RootCertStore, ServerConfig};
use serde::Serialize;
use std::{fs::File, io, net::SocketAddr, sync::Arc};
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

    let total_route = health_get.or(get).or(post);

    // Build TLS configuration.
    let tls_config = {
        let cert_path = config.get_str("tls.server.certificate.path").unwrap();
        let key_path = config.get_str("tls.server.key.path").unwrap();
        let client_ca_path = config.get_str("tls.client.ca.path").unwrap();

        let mut rcs = RootCertStore::empty();
        let file = File::open(&client_ca_path).unwrap();
        let mut reader = io::BufReader::new(file);
        rcs.add_pem_file(&mut reader)
            .map_err(|_err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to load root cert store from {}", &client_ca_path),
                )
            })
            .unwrap();

        let mut config = ServerConfig::new(AllowAnyAuthenticatedClient::new(rcs));
        // Select a certificate to use.
        let file = File::open(&cert_path).unwrap();
        let mut reader = io::BufReader::new(file);
        let certs = pemfile::certs(&mut reader)
            .map_err(|_err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Cannot load certificate from {}", &cert_path),
                )
            })
            .unwrap();
        let file = File::open(&key_path).unwrap();
        let mut reader = io::BufReader::new(file);
        let keys = pemfile::pkcs8_private_keys(&mut reader)
            .map_err(|_err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Cannot load private key from {}", &key_path),
                )
            })
            .unwrap();
        let key = keys
            .into_iter()
            .next()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("No keys found in the private key file {}", key_path),
                )
            })
            .unwrap();
        config
            .set_single_cert(certs, key)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{}", err)))
            .unwrap();
        Arc::new(config)
    };

    let socket_addr: SocketAddr = ([0, 0, 0, 0], port).into();
    loop {
        if let Err(e) =
            bootstrap::serve_mtls(socket_addr, tls_config.clone(), total_route.clone()).await
        {
            eprintln!("Problem accepting TLS connection: {}", e);
        }
    }
}
