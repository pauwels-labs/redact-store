mod routes;
mod traverse;

use redact_crypto::MongoKeyStorer;
use redact_data::MongoDataStorer;
use redact_config::Configurator;
use serde::Serialize;
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
    let data_storer = MongoDataStorer::new(&db_url, &db_name).await;
    let key_storer = MongoKeyStorer::new(&db_url, &db_name).await;

    // Build out routes
    let health_get = warp::path!("healthz")
        .and(warp::get())
        .map(|| warp::reply::json(&Healthz {}));

    let keys_get = warp::path("keys")
        .and(warp::get())
        .and(routes::keys::get(key_storer.clone()).or(routes::keys::list(key_storer.clone())));
    let keys_post = warp::path("keys")
        .and(warp::post())
        .and(routes::keys::create(key_storer.clone()));

    let data_get = warp::path("data")
        .and(warp::get())
        .and(routes::data::get(data_storer.clone()));
    let data_post = warp::path("data")
        .and(warp::post())
        .and(routes::data::create(data_storer.clone()));

    let routes = health_get
        .or(keys_get)
        .or(keys_post)
        .or(data_get)
        .or(data_post)
        .with(warp::log("routes"));

    // Start the server
    println!("starting server listening on ::{}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
