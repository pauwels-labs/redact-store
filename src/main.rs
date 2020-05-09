use futures::future;
use hyper::server::conn::AddrIncoming;
use rust_config::Configurator;
use std::convert::Infallible;
use warp::Filter;

pub mod tls;

use tls::{TlsAcceptor, TlsConfigBuilder, Transport};

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
    let filter = warp::service(routes);
    let make_svc = hyper::service::make_service_fn(move |transport| {
        // match transport.state {
        //     State::Handshaking(_) => {
        //         println!("in handshake");
        //         future::ok::<_, Infallible>(filter)
        //     }
        //     State::Streaming(ref stream) => {
        //         let (_incoming, sess) = stream.get_ref();
        //         println!("in streaming, sni = {}", sess.get_sni_hostname().unwrap());
        //         future::ok::<_, Infallible>(filter)
        //     }
        // }
        // future::ok::<_, Infallible>(filter)
        // let filter = filter.clone();
        // match transport.state {
        //     State::Handshaking(_) => Ok::<_, Infallible>(filter),
        //     State::Streaming(_) => Ok::<_, Infallible>(filter),
        // }
        let inner = filter.clone();
        let remote_addr = Transport::remote_addr(transport);
        let peer_certificates = Transport::peer_certificates(transport);
        future::ok::<_, Infallible>(hyper::service::service_fn(move |req| {
            inner.call(req, remote_addr)
        }))
    });

    let tls_config = TlsConfigBuilder::new()
        .key_path(tls_key_path)
        .cert_path(tls_cert_path)
        .client_ca_path(tls_client_ca_path)
        .build()
        .unwrap();
    let incoming = AddrIncoming::bind(&(([127, 0, 0, 1], 8080).into()))
        .and_then(|mut i| {
            i.set_nodelay(true);
            Ok(i)
        })
        .unwrap();
    hyper::Server::builder(TlsAcceptor::new(tls_config, incoming))
        .serve(make_svc)
        .await
        .unwrap();
}
