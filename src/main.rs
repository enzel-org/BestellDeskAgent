// BestellDeskAgent/src/main.rs
use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    mongo_uri: Arc<String>,
}

#[derive(Serialize)]
struct ResolveResponse {
    mongo_uri: String,
}

async fn resolve_handler(State(st): State<AppState>) -> Json<ResolveResponse> {
    Json(ResolveResponse {
        mongo_uri: st.mongo_uri.as_ref().clone(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read settings from environment (provided by systemd)
    let mongo_uri = env::var("MONGODB_URI").map_err(|_| {
        anyhow::anyhow!(
            "MONGODB_URI is not set. Provide it via systemd Environment= or EnvironmentFile="
        )
    })?;

    // Optional bind address (default 0.0.0.0:8443)
    let bind = env::var("AGENT_BIND").unwrap_or_else(|_| "0.0.0.0:8443".to_string());
    let addr: SocketAddr = bind.parse().map_err(|e| {
        anyhow::anyhow!("Invalid AGENT_BIND '{}': {e}", bind)
    })?;

    let state = AppState {
        mongo_uri: Arc::new(mongo_uri),
    };

    let app = Router::new()
        .route("/v1/mongo-uri", get(resolve_handler))
        .with_state(state);

    let listener = TcpListener::bind(addr).await?;
    println!("Agent running on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
