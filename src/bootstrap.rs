use futures::TryFutureExt;
use std::{io, sync::Arc, time::SystemTime};
use tokio::net;
use tokio_rustls::{
    rustls::{
        server::{ClientCertVerified, ClientCertVerifier},
        Certificate, ServerConfig,
    },
    TlsAcceptor,
};
use warp::hyper::service::{self, Service};

pub struct AllowAnyClient {}
impl ClientCertVerifier for AllowAnyClient {
    fn client_auth_root_subjects(&self) -> Option<tokio_rustls::rustls::DistinguishedNames> {
        Some(vec![])
    }

    fn verify_client_cert(
        &self,
        _: &tokio_rustls::rustls::Certificate,
        _: &[tokio_rustls::rustls::Certificate],
        _: SystemTime,
    ) -> Result<ClientCertVerified, tokio_rustls::rustls::Error> {
        Ok(ClientCertVerified::assertion())
    }
}

pub async fn serve_mtls<F>(
    listener: &net::TcpListener,
    tls_config: Arc<ServerConfig>,
    warp_filter: F,
) -> io::Result<()>
where
    F: warp::Filter + Clone + Send + Sync + 'static,
    <F::Future as futures::TryFuture>::Ok: warp::Reply,
{
    let tls_acceptor = TlsAcceptor::from(tls_config);

    // Wait for an incoming TCP connection
    let (socket, _) = listener.accept().await?;

    // Interpret data coming through the TCP stream as a TLS stream
    let stream = tls_acceptor
        .accept(socket)
        .map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Problem accepting TLS connection: {:?}", err),
            )
        })
        .await?;

    // Hand off actual request handling to a new tokio task
    tokio::task::spawn(async move {
        // Pull the client certificate out of the TLS session
        let (_, server_connection) = stream.get_ref();
        let client_cert = server_connection.peer_certificates().and_then(|certs| {
            if certs.is_empty() {
                None
            } else {
                Some(certs[0].clone())
            }
        });

        // Turn the warp filter into a service, but instead of using that
        // service directly as usual, we wrap it around another service
        // so that we can modify the request and inject the client certificate
        // into the request extentions before it goes into the filter.
        let mut svc = warp::service(warp_filter.clone());
        let service = service::service_fn(move |mut req| {
            if let Some(cert) = client_cert.to_owned() {
                req.extensions_mut().insert(cert);
            }
            svc.call(req)
        });
        if let Err(e) = hyper::server::conn::Http::new()
            .serve_connection(stream, service)
            .await
        {
            eprintln!("Error handling request: {}", e);
        }
    });

    Ok(())
}

pub async fn serve_xfcc<F>(listener: &net::TcpListener, warp_filter: F) -> io::Result<()>
where
    F: warp::Filter + Clone + Send + Sync + 'static,
    <F::Future as futures::TryFuture>::Ok: warp::Reply,
{
    // Wait for an incoming TCP connection
    let (socket, _) = listener.accept().await?;

    // Hand off actual request handling to a new tokio task
    tokio::task::spawn(async move {
        // Turn the warp filter into a service, but instead of using that
        // service directly as usual, we wrap it around another service
        // so that we can modify the request and inject the client certificate
        // into the request extentions before it goes into the filter.
        let mut svc = warp::service(warp_filter.clone());
        let service = service::service_fn(move |mut req| {
            let cert = req
                .headers()
                .get("x-forwarded-client-cert")
                .and_then(|xfcc_header| xfcc_header.to_str().ok())
                .and_then(|xfcc_header_str| {
                    xfcc_header_str
                        .split(';')
                        .find(|&elem| &elem[0..5] == "Cert=")
                })
                .and_then(|encoded_cert| {
                    urlencoding::decode(&encoded_cert[6..encoded_cert.len() - 1]).ok()
                })
                .and_then(|cert| pem::parse(cert.into_owned()).ok());

            if let Some(cert) = cert {
                req.extensions_mut()
                    .insert(Certificate(cert.into_contents()));
            }

            svc.call(req)
        });
        if let Err(e) = hyper::server::conn::Http::new()
            .serve_connection(socket, service)
            .await
        {
            eprintln!("Error handling request: {}", e);
        }
    });

    Ok(())
}
