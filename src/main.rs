// agent/src/main.rs
use axum::{routing::get, Router, response::IntoResponse, Json, http::StatusCode, extract::State};
use serde::Serialize;
use std::{fs, net::SocketAddr, sync::Arc};
use anyhow::{Context, Result};
use rustls::{pki_types::CertificateDer, pki_types::PrivateKeyDer, ServerConfig};
use tokio_rustls::TlsAcceptor;

#[derive(Clone)]
struct AppState {
    mongo_uri: String,
    api_key: String, // shared secret
}

#[derive(Serialize)]
struct MongoResp { uri: String }

#[tokio::main]
async fn main() -> Result<()> {
    // --- Read secrets/paths from env or default paths ---
    let mongo_uri = std::env::var("MONGODB_URI")
        .or_else(|_| fs::read_to_string("/etc/bestelldesk/mongodb_uri"))
        .context("MONGODB_URI not set and /etc/bestelldesk/mongodb_uri not readable")?
        .trim().to_string();

    // Single shared API key (rotate regularly)
    let api_key = std::env::var("API_KEY")
        .or_else(|_| fs::read_to_string("/etc/bestelldesk/api_key"))
        .context("API_KEY missing (env or /etc/bestelldesk/api_key)")?
        .trim().to_string();

    let cert_path  = std::env::var("TLS_CERT").unwrap_or("/etc/bestelldesk/tls/server.crt".into());
    let key_path   = std::env::var("TLS_KEY").unwrap_or("/etc/bestelldesk/tls/server.key".into());
    let bind_addr  = std::env::var("BIND").unwrap_or("0.0.0.0:8443".into());

    // --- Load TLS materials (server-only TLS) ---
    let server_cert = fs::read(cert_path)?;
    let server_key  = fs::read(key_path)?;

    let certs = rustls_pemfile::certs(&mut &*server_cert)?
        .into_iter().map(CertificateDer::from).collect::<Vec<_>>();
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut &*server_key)?;
    if keys.is_empty() {
        keys = rustls_pemfile::rsa_private_keys(&mut &*server_key)?;
    }
    let key = PrivateKeyDer::from(keys.into_iter().next().context("no private key found")?);

    // No client auth
    let tls_config = ServerConfig::builder().with_no_client_auth().with_single_cert(certs, key)?;
    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

    let state = AppState { mongo_uri, api_key };

    let app = Router::new()
        .route("/v1/mongo-uri", get(get_uri))
        .with_state(state);

    let addr: SocketAddr = bind_addr.parse()?;
    println!("Agent listening on https://{addr}");
    axum_server::bind_rustls(addr, tls_acceptor).serve(app.into_make_service()).await?;
    Ok(())
}

async fn get_uri(
    State(st): State<AppState>,
    axum::http::HeaderMap,
) -> impl IntoResponse {
    // Lightweight auth: expect header X-API-Key: <secret>
    // You can switch to Authorization: Bearer <token> if preferred.
    let key = axum::http::HeaderName::from_static("x-api-key");
    let Some(h) = HeaderMap.get(&key) else {
        return (StatusCode::UNAUTHORIZED, "missing X-API-Key").into_response();
    };
    if h.to_str().ok() != Some(st.api_key.as_str()) {
        return (StatusCode::UNAUTHORIZED, "invalid api key").into_response();
    }

    Json(MongoResp { uri: st.mongo_uri.clone() }).into_response()
}

// Same axum_server helper as before…
mod axum_server {
    use axum::serve;
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;
    use std::net::SocketAddr;
    use hyper::server::conn::Http;

    pub async fn bind_rustls<S>(addr: SocketAddr, tls: TlsAcceptor, svc: S) -> anyhow::Result<()>
    where
        S: tower::make::MakeService<(), hyper::Request<hyper::body::Incoming>, Response = hyper::Response<axum::body::Body>, Error = std::convert::Infallible> + Clone + Send + 'static,
        S::Service: Send + 'static,
    {
        let listener = TcpListener::bind(addr).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            let tls = tls.clone();
            let svc = svc.clone();
            tokio::spawn(async move {
                let Ok(tls_stream) = tls.accept(stream).await else { return; };
                let _ = Http::new().serve_connection(tls_stream, svc).await;
            });
        }
    }
}
