mod bootstrap;
mod error_handler;
mod routes;

use crate::error_handler::handle_rejection;
use chrono::{prelude::*, Duration};
use der::asn1::{Any, OctetString};
use pkcs8::{PrivateKeyDocument, PrivateKeyInfo};
use redact_config::Configurator;
use redact_crypto::{
    key::sodiumoxide::{
        SodiumOxideEd25519SecretAsymmetricKey, SodiumOxideEd25519SecretAsymmetricKeyBuilder,
    },
    storage::gcs::GoogleCloudStorer,
    Builder, HasAlgorithmIdentifier, HasByteSource, PublicAsymmetricKey,
};
use redact_crypto::{storage::NonIndexedTypeStorer::GoogleCloud, HasPublicKey};
use redact_crypto::{x509::DistinguishedName, MongoStorer, TypeStorer};
use rustls::{internal::pemfile, AllowAnyAuthenticatedClient, RootCertStore, ServerConfig};
use serde::Serialize;
use std::{
    convert::TryInto,
    fs::File,
    io::{self, ErrorKind, Read, Write},
    net::SocketAddr,
    sync::Arc,
};
use warp::Filter;

#[derive(Serialize)]
struct Healthz {}

#[tokio::main]
async fn main() {
    // pretty_env_logger::init();
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

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

    let generate_crypto_material = config.get_bool("tls.generate").unwrap();
    if generate_crypto_material {
        // Make the storer TLS CA key if it doesn't exist
        let ca_key_path = config.get_str("tls.ca.key.path").unwrap();
        let ca_key = match File::open(&ca_key_path) {
            Ok(mut f) => {
                let mut pem = String::new();
                f.read_to_string(&mut pem).unwrap();
                let pkd = PrivateKeyDocument::from_pem(&pem).unwrap();
                let seed_bytes: OctetString =
                    TryInto::<Any>::try_into(pkd.private_key_info().private_key)
                        .unwrap()
                        .try_into()
                        .unwrap();
                // TODO(ajpauwels): Add a match on the AlgorithmIdentifier within the PEM file to
                //                  determine the proper key type; assuming NaCl Ed25519 for now
                let builder = SodiumOxideEd25519SecretAsymmetricKeyBuilder {};
                Ok(builder.build(Some(seed_bytes.as_bytes())).unwrap())
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(SodiumOxideEd25519SecretAsymmetricKey::new()),
                _ => Err(e),
            },
        }
        .unwrap();

        // Make the storer TLS CA cert and PKCS12 file if it doesn't exist
        let ca_cert_o = config.get_str("tls.ca.certificate.o").unwrap();
        let ca_cert_ou = config.get_str("tls.ca.certificate.ou").unwrap();
        let ca_cert_cn = config.get_str("tls.ca.certificate.cn").unwrap();
        let ca_cert_dn = DistinguishedName {
            o: &ca_cert_o,
            ou: &ca_cert_ou,
            cn: &ca_cert_cn,
        };
        if let Err(e) = File::open(config.get_str("tls.ca.certificate.path").unwrap()) {
            match e.kind() {
                ErrorKind::NotFound => {
                    let not_before = Utc::now();
                    let not_after = not_before
                        + Duration::days(config.get_int("tls.ca.certificate.expires_in").unwrap());
                    let tls_cert = redact_crypto::cert::setup_cert::<_, PublicAsymmetricKey>(
                        &ca_key,
                        None,
                        &ca_cert_dn,
                        None,
                        not_before,
                        not_after,
                        true,
                        None,
                    )
                    .unwrap();
                    let mut tls_cert_vec: Vec<u8> = vec![];
                    let mut tls_cert_file =
                        File::create(config.get_str("tls.ca.certificate.path").unwrap()).unwrap();
                    tls_cert_vec
                        .write_all(b"-----BEGIN CERTIFICATE-----\n")
                        .unwrap();
                    base64::encode(tls_cert)
                        .as_bytes()
                        .chunks(64)
                        .for_each(|chunk| {
                            tls_cert_vec.write_all(chunk).unwrap();
                            tls_cert_vec.write_all(b"\n").unwrap();
                        });
                    tls_cert_vec
                        .write_all(b"-----END CERTIFICATE-----\n")
                        .unwrap();
                    tls_cert_file.write_all(&tls_cert_vec).unwrap();

                    let storer_tls_key_bs = ca_key.byte_source();
                    let mut storer_tls_key_bytes = vec![0x04, 0x20];
                    storer_tls_key_bytes
                        .extend_from_slice(&storer_tls_key_bs.get().unwrap()[0..32]);
                    let storer_tls_key_pkcs8 =
                        PrivateKeyInfo::new(ca_key.algorithm_identifier(), &storer_tls_key_bytes);
                    let mut pkcs8_file = File::create(&ca_key_path).unwrap();
                    pkcs8_file
                        .write_all((*storer_tls_key_pkcs8.to_pem()).as_bytes())
                        .unwrap();
                }
                _ => Err(e).unwrap(),
            }
        }

        // Make the storer client TLS key if it doesn't exist
        let storer_key_path = config.get_str("tls.server.key.path").unwrap();
        let ca_key = match File::open(&storer_key_path) {
            Ok(mut f) => {
                let mut pem = String::new();
                f.read_to_string(&mut pem).unwrap();
                let pkd = PrivateKeyDocument::from_pem(&pem).unwrap();
                let seed_bytes: OctetString =
                    TryInto::<Any>::try_into(pkd.private_key_info().private_key)
                        .unwrap()
                        .try_into()
                        .unwrap();
                // TODO(ajpauwels): Add a match on the AlgorithmIdentifier within the PEM file to
                //                  determine the proper key type; assuming NaCl Ed25519 for now
                let builder = SodiumOxideEd25519SecretAsymmetricKeyBuilder {};
                Ok(builder.build(Some(seed_bytes.as_bytes())).unwrap())
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(SodiumOxideEd25519SecretAsymmetricKey::new()),
                _ => Err(e),
            },
        }
        .unwrap();

        // Make the storer TLS cert and PKCS12 file if it doesn't exist
        if let Err(e) = File::open(config.get_str("tls.server.certificate.path").unwrap()) {
            match e.kind() {
                ErrorKind::NotFound => {
                    let storer_cert_o = config.get_str("tls.server.certificate.o").unwrap();
                    let storer_cert_ou = config.get_str("tls.server.certificate.ou").unwrap();
                    let storer_cert_cn = config.get_str("tls.server.certificate.cn").unwrap();
                    let storer_cert_dn = DistinguishedName {
                        o: &storer_cert_o,
                        ou: &storer_cert_ou,
                        cn: &storer_cert_cn,
                    };
                    let not_before = Utc::now();
                    let not_after = not_before
                        + Duration::days(
                            config.get_int("tls.server.certificate.expires_in").unwrap(),
                        );
                    let storer_key = SodiumOxideEd25519SecretAsymmetricKey::new();
                    let tls_cert = redact_crypto::cert::setup_cert(
                        &ca_key,
                        Some(&storer_key.public_key().unwrap()),
                        &ca_cert_dn,
                        Some(&storer_cert_dn),
                        not_before,
                        not_after,
                        false,
                        Some(&["localhost"]),
                    )
                    .unwrap();
                    let mut tls_cert_vec: Vec<u8> = vec![];
                    let mut tls_cert_file =
                        File::create(config.get_str("tls.server.certificate.path").unwrap())
                            .unwrap();
                    tls_cert_vec
                        .write_all(b"-----BEGIN CERTIFICATE-----\n")
                        .unwrap();
                    base64::encode(tls_cert)
                        .as_bytes()
                        .chunks(64)
                        .for_each(|chunk| {
                            tls_cert_vec.write_all(chunk).unwrap();
                            tls_cert_vec.write_all(b"\n").unwrap();
                        });
                    tls_cert_vec
                        .write_all(b"-----END CERTIFICATE-----\n")
                        .unwrap();
                    tls_cert_file.write_all(&tls_cert_vec).unwrap();

                    let storer_tls_key_bs = storer_key.byte_source();
                    let mut storer_tls_key_bytes = vec![0x04, 0x20];
                    storer_tls_key_bytes
                        .extend_from_slice(&storer_tls_key_bs.get().unwrap()[0..32]);
                    let storer_tls_key_pkcs8 = PrivateKeyInfo::new(
                        storer_key.algorithm_identifier(),
                        &storer_tls_key_bytes,
                    );
                    let mut pkcs8_file = File::create(&storer_key_path).unwrap();
                    pkcs8_file
                        .write_all((*storer_tls_key_pkcs8.to_pem()).as_bytes())
                        .unwrap();
                }
                _ => Err(e).unwrap(),
            }
        }
    }

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

    let total_route = health_get
        .or(get)
        .or(post)
        .with(warp::log("routes"))
        .recover(handle_rejection);

    // Build TLS configuration.
    let tls_config = {
        let cert_path = config.get_str("tls.server.certificate.path").unwrap();
        let client_ca_path = config.get_str("tls.client.ca.path").unwrap();

        let mut rcs = RootCertStore::empty();
        if let Ok(file) = File::open(&client_ca_path) {
            let mut reader = io::BufReader::new(file);
            rcs.add_pem_file(&mut reader)
                .map_err(|_err| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to load root cert store from {}", &client_ca_path),
                    )
                })
                .unwrap();
        }

        let mut server_config = ServerConfig::new(AllowAnyAuthenticatedClient::new(rcs));
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
        let storer_key_path = config.get_str("tls.server.key.path").unwrap();
        let file = File::open(&storer_key_path).unwrap();
        let mut reader = io::BufReader::new(file);
        let keys = pemfile::pkcs8_private_keys(&mut reader)
            .map_err(|_err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Cannot load private key from {}", &storer_key_path),
                )
            })
            .unwrap();
        let key = keys
            .into_iter()
            .next()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("No keys found in the private key file {}", storer_key_path),
                )
            })
            .unwrap();
        server_config
            .set_single_cert(certs, key)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{}", err)))
            .unwrap();
        Arc::new(server_config)
    };

    let socket_addr: SocketAddr = ([0, 0, 0, 0], port).into();
    println!("starting server listening on ::{}", port);
    loop {
        if let Err(e) =
            bootstrap::serve_mtls(socket_addr, tls_config.clone(), total_route.clone()).await
        {
            eprintln!("Problem accepting TLS connection: {}", e);
        }
    }
}
