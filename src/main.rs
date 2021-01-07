pub mod traverse;

use mongodb::{options::ClientOptions, Client};
use rust_config::Configurator;
use serde::Serialize;
use warp::Filter;

#[derive(Serialize)]
struct Healthz {}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Extract config with a REDACT_ env var prefix
    let config = rust_config::new("REDACT").unwrap();

    // Determine port to listen on
    let port = match config.get_int("server.port") {
        Ok(port) => {
            if port < 1 || port > 65535 {
                println!("listen port value '{}' is not between 1 and 65535", port);
                8080 as u16
            } else {
                port as u16
            }
        }
        Err(e) => {
            match e {
                // Suppress debug logging if server.port was simply not set
                rust_config::ConfigError::NotFound(_) => (),
                _ => println!("{}", e),
            }
            8080 as u16
        }
    };

    // Extract handle to the database
    let db_url = config.get_str("db.url").unwrap();
    let db_client_options = ClientOptions::parse_with_resolver_config(
        &db_url,
        mongodb::options::ResolverConfig::cloudflare(),
    )
    .await
    .unwrap();
    let db_client = Client::with_options(db_client_options).unwrap();

    let db_name = config.get_str("db.name").unwrap();
    let db = db_client.database(&db_name);

    // Initial ping to establish DB connection
    println!("connecting to database");
    db.clone()
        .run_command(bson::doc! {"ping": 1}, None)
        .await
        .unwrap();
    println!("connected to database");

    // Build out routes
    let health_route = warp::path!("healthz").map(|| warp::reply::json(&Healthz {}));
    let data_routes = filters::data(db);

    // Start the server
    println!("starting server");
    let routes = health_route.or(data_routes).with(warp::log("routes"));

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

mod filters {
    use mongodb::Database;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};

    #[derive(Serialize)]
    struct GetResponse {
        data_type: String,
        path: String,
        value: serde_json::Value,
    }

    #[derive(Serialize)]
    struct CreateResponse {
        success: bool,
        msg: String,
    }

    #[derive(Serialize, Deserialize)]
    struct GetQuery {
        path: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Data {
        pub data_type: String,
        pub path: String,
        pub value: Value,
    }

    #[derive(Debug)]
    struct NotFound;
    impl Reject for NotFound {}

    pub fn data(db: Database) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        data_get(db.clone()).or(data_create(db.clone()))
    }

    pub fn data_get(db: Database) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("data")
            .and(warp::get())
            .and(warp::query::<GetQuery>())
            .and(with_db(db))
            .and_then(move |query: GetQuery, db: Database| async move {
                let filter_options = mongodb::options::FindOneOptions::builder().build();
                let filter = bson::doc! { "path": query.path };

                match db.collection("data").find_one(filter, filter_options).await {
                    Ok(Some(doc)) => {
                        let data: Data = bson::from_document(doc).unwrap();
                        Ok(warp::reply::json(&data))
                    }
                    Ok(None) => Err(warp::reject::custom(NotFound)),
                    Err(e) => Err(warp::reject::reject()),
                }
            })
            .recover(move |rejection: Rejection| async move {
                let reply = warp::reply::reply();

                if let Some(NotFound) = rejection.find() {
                    Ok(warp::reply::with_status(reply, StatusCode::NOT_FOUND))
                } else {
                    Ok(warp::reply::with_status(
                        reply,
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            })
    }

    pub fn data_create(
        db: Database,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("data")
            .and(warp::post())
            .and(warp::body::content_length_limit(1024 * 1000 * 250))
            .and(warp::body::json::<Data>())
            .and(with_db(db))
            .and_then(move |contents: Data, db: Database| async move {
                match super::traverse::insert(&db, &contents).await {
                    Ok(_) => Ok(warp::reply::json(&CreateResponse {
                        success: true,
                        msg: "inserted".to_string(),
                    })),
                    Err(_) => Err(warp::reject::reject()),
                }
            })
    }

    fn with_db(
        db: Database,
    ) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}
