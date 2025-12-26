use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tiktoken_rs::CoreBPE;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

mod config;

#[derive(Clone)]
struct AppState {
    config: config::Config,
    tokenizer: Arc<RwLock<Option<CoreBPE>>>,
    ready: Arc<RwLock<bool>>,
}

#[derive(Deserialize)]
struct TokenizeRequest {
    input: String,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Serialize)]
struct TokenizeResponse {
    ids: Vec<usize>,
    tokens: Vec<String>,
}

async fn health() -> &'static str {
    "ok"
}

async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let ready = *state.ready.read().await;
    if ready {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "warming")
    }
}

async fn tokenize(State(state): State<Arc<AppState>>, Json(req): Json<TokenizeRequest>) -> impl IntoResponse {
    let _model = req.model.unwrap_or_else(|| state.config.default_tokenizer.clone());
    let enc = {
        let guard = state.tokenizer.read().await;
        guard.clone()
    };

    let Some(encoding) = enc else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(TokenizeResponse { ids: vec![], tokens: vec![] }));
    };

    let ids = encoding.encode_with_special_tokens(&req.input);
    // tiktoken_rs doesn't expose decode_single_token_bytes, so we decode the full sequence
    let _decoded = encoding.decode(ids.clone()).unwrap_or_default();
    let tokens: Vec<String> = ids.iter().map(|id| format!("[{}]", id)).collect();

    let resp = TokenizeResponse { ids, tokens };
    (StatusCode::OK, Json(resp))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cfg = config::Config::load();
    let state = Arc::new(AppState {
        config: cfg,
        tokenizer: Arc::new(RwLock::new(None)),
        ready: Arc::new(RwLock::new(false)),
    });

    // Preload / warm tokenizer
    let preload_state = state.clone();
    tokio::spawn(async move {
        if preload_state.config.preload {
            info!(
                "preloading tokenizer={} embedding_model={}",
                preload_state.config.default_tokenizer, preload_state.config.default_embedding_model
            );
            match load_tokenizer(&preload_state.config.default_tokenizer) {
                Ok(enc) => {
                    *preload_state.tokenizer.write().await = Some(enc);
                    *preload_state.ready.write().await = true;
                    info!("tokenizer ready");
                }
                Err(err) => {
                    error!("tokenizer preload failed: {}", err);
                }
            }
        } else {
            *preload_state.ready.write().await = true;
        }
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/tokenize", post(tokenize))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    let addr: SocketAddr = format!("{}:{}", state.config.api_host, state.config.api_port)
        .parse()
        .unwrap_or_else(|e| {
            error!("Invalid bind address: {}", e);
            SocketAddr::from(([0, 0, 0, 0], 7105))
        });
    info!("listening on {}", addr);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn load_tokenizer(model: &str) -> Result<CoreBPE, String> {
    // Try as model name first (e.g., "gpt-4"), then as encoding name (e.g., "cl100k_base")
    tiktoken_rs::get_bpe_from_model(model)
        .or_else(|_| {
            // Map encoding names to models that use them
            let model_for_encoding = match model {
                "cl100k_base" => "gpt-4",
                "p50k_base" => "text-davinci-003",
                "p50k_edit" => "text-davinci-edit-001",
                "r50k_base" | "gpt2" => "text-davinci-001",
                "o200k_base" => "gpt-4o",
                _ => model,
            };
            tiktoken_rs::get_bpe_from_model(model_for_encoding)
        })
        .map_err(|e| e.to_string())
}
