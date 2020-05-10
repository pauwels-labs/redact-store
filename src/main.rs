use rust_config::Configurator;
use warp::Filter;

#[tokio::main]
async fn main() {
    let config = rust_config::new().unwrap();
    let tls_key_path = config.get_str("tls.server.key.path").unwrap();
    let tls_cert_path = config.get_str("tls.server.cert.path").unwrap();
    let tls_client_ca_path = config.get_str("tls.client.ca.path").unwrap();

    let healthz = warp::path!("healthz")
        .and(warp::tls::peer_certificates())
        .map(|certs| {
            println!("{:?}", certs);
            "ok\n"
        });
    let apiv1 = warp::path!("api" / "v1").map(|| "ok\n");

    let routes = warp::get().and(healthz.or(apiv1));
    warp::serve(routes)
        .tls()
        .key_path(tls_key_path)
        .cert_path(tls_cert_path)
        .client_ca_path(tls_client_ca_path)
        .run(([127, 0, 0, 1], 8080))
        .await;
}
