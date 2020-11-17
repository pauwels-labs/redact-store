pub mod traverse;

use mongodb::{options::ClientOptions, Client};
use rust_config::Configurator;
use serde::Serialize;
use warp::Filter;

#[derive(Serialize)]
struct Healthz {}

#[derive(Serialize)]
struct StoreResponse {
    success: bool,
    msg: String,
}

#[tokio::main]
async fn main() {
    let config = rust_config::new("REDACT").unwrap();
    let port = match config.get_int("server.port") {
        Ok(port) => {
            if port < 1 || port > 65535 {
                // TODO: Add debug log entry here
                8080 as u16
            } else {
                port as u16
            }
        }
        Err(_) => 8080 as u16,
    };

    let db_url = config.get_str("db.url").unwrap();
    let db_client_options = ClientOptions::parse_with_resolver_config(
        &db_url,
        mongodb::options::ResolverConfig::cloudflare(),
    )
    .await
    .unwrap();
    let db_client = Client::with_options(db_client_options).unwrap();
    let db = db_client.database("redact");

    // Initial ping to establish DB connection
    println!("connecting to database");
    db.run_command(bson::doc! {"ping": 1}, None).await.unwrap();
    println!("connected to database");

    let health_route = warp::path!("healthz").map(|| warp::reply::json(&Healthz {}));
    let store_route = warp::path!("store")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 1000 * 250))
        .and(warp::body::json())
        .and_then(move |contents| {
            let db = db.clone();
            async move {
                match traverse::traverse(&db, "", &contents).await {
                    Ok(_) => {
                        return Ok(warp::reply::json(&StoreResponse {
                            success: true,
                            msg: "inserted".to_string(),
                        }))
                    }
                    Err(_) => return Err(warp::reject::reject()),
                }
            }
        });

    println!("starting server");
    let all_routes = health_route.or(store_route);
    warp::serve(all_routes).run(([0, 0, 0, 0], port)).await;
}
