use std::net::SocketAddr;

use axum::extract::FromRef;
use axum::{routing::get, Router};
use tower_http::services::{ServeDir, ServeFile};

use crate::app::{SharedPaused, SharedWorld};
use crate::http::endpoints::date::{date, set_pause};
use crate::http::endpoints::home::home;

/// State shared with every handler. `FromRef` lets handlers extract just
/// the piece they need instead of the whole struct.
#[derive(Clone)]
pub struct HttpState {
    pub world: SharedWorld,
    pub is_tick_paused: SharedPaused,
}

impl FromRef<HttpState> for SharedWorld {
    fn from_ref(state: &HttpState) -> Self {
        SharedWorld::clone(&state.world)
    }
}

impl FromRef<HttpState> for SharedPaused {
    fn from_ref(state: &HttpState) -> Self {
        SharedPaused::clone(&state.is_tick_paused)
    }
}

/// Build the full router: `/api/v1/*` → axum routes; everything else →
/// static files from `static_dir` with `index.html` fallback for SPA routing.
pub fn router(state: HttpState, static_dir: &'static str) -> Router {
    let api = Router::new()
        .route("/api/v1/home", get(home))
        .route("/api/v1/date", get(date).post(set_pause))
        .with_state(state);

    let index = format!("{static_dir}/index.html");
    let serve_dir = ServeDir::new(static_dir).fallback(ServeFile::new(index));

    api.fallback_service(serve_dir)
}

/// Spawn the HTTP server on its own OS thread with its own tokio runtime.
/// Binds `MEDIEVAL_HTTP_ADDR` (default `127.0.0.1:7777`) and serves SPA routes.
pub fn startup(world: SharedWorld, is_tick_paused: SharedPaused) {
    // Bind address for the HTTP server. Override with MEDIEVAL_HTTP_ADDR.
    let addr: SocketAddr = std::env::var("MEDIEVAL_HTTP_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:7777".into())
        .parse()
        .expect("invalid MEDIEVAL_HTTP_ADDR (expected e.g. 127.0.0.1:7777)");

    // Directory where the built SPA lives. Override with MEDIEVAL_STATIC_DIR.
    let static_dir: &'static str = Box::leak(
        std::env::var("MEDIEVAL_STATIC_DIR").unwrap_or_else(|_| "web/dist".into())
            .into_boxed_str(),
    );

    let state = HttpState { world, is_tick_paused };

    std::thread::Builder::new()
        .name("medieval-http".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build tokio runtime");
            rt.block_on(async move {
                let listener =
                    tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|e| {
                        panic!("failed to bind {}: {}", addr, e)
                    });
                println!("HTTP server listening on http://{}", addr);
                axum::serve(listener, router(state, static_dir))
                    .await
                    .expect("HTTP server crashed");
            });
        })
        .expect("failed to spawn HTTP thread");
}
