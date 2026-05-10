pub mod ws;

use std::sync::Arc;
use axum::{
    Router,
    routing::get,
    extract::State as AxumState,
    response::{IntoResponse, Json},
    http::{header, StatusCode, Uri},
};
use tower_http::cors::CorsLayer;
use include_dir::{include_dir, Dir};
use crate::state::{AppState, LightingState};

static DIST: Dir<'_> = include_dir!("../dist");

const DIST_NOT_FOUND_HTML: &str = r#"<!DOCTYPE html>
<html><head><title>VRC Lighting Controller</title>
<style>body{background:#1a1a2e;color:#e0e0e0;font-family:system-ui;display:flex;align-items:center;justify-content:center;height:100vh;margin:0}
.msg{text-align:center}h1{color:#e94560}
</style></head><body>
<div class="msg"><h1>VRC Lighting Controller</h1>
<p>Web UI dist files not found. Build the frontend first.</p>
<p><code>pnpm build</code></p></div>
</body></html>"#;

pub async fn run(state: Arc<AppState>) {
    let port = {
        let lighting = state.lighting.read();
        lighting.config.web_port
    };

    let app = Router::new()
        .route("/api/state", get(get_state))
        .route("/ws", get(ws::ws_handler))
        .layer(CorsLayer::permissive())
        .fallback(serve_dist)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("failed to bind web server");

    log::info!("Web server listening on 0.0.0.0:{}", port);

    axum::serve(listener, app)
        .await
        .expect("web server error");
}

async fn serve_dist(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if let Some(file) = DIST.get_file(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref().to_string())],
            file.contents(),
        );
    }

    // SPA fallback
    if let Some(index) = DIST.get_file("index.html") {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html".to_string())],
            index.contents(),
        );
    }

    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "text/html".to_string())],
        DIST_NOT_FOUND_HTML.as_bytes(),
    )
}

async fn get_state(
    AxumState(state): AxumState<Arc<AppState>>,
) -> Json<LightingState> {
    let lighting = state.lighting.read().clone();
    Json(lighting)
}
