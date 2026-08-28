use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{extract::State, routing::get, Json, Router};
use hecs::World;
use serde_json::{json, Value};
use tower_http::services::{ServeDir, ServeFile};

use crate::components::core::StringId;
use crate::components::land::{LandBorders, LandHolding, LandName, LandTerrain};
use crate::components::road::{RoadBetween, RoadDistanceDays, RoadPoints};
use crate::components::settlement::{SettlementLandId, SettlementPopulation};

/// A handle to the shared ECS world used by HTTP handlers.
///
/// The same `Arc<Mutex<World>>` is owned by `App` and cloned into the HTTP
/// thread so handlers always see the latest entity state.
pub type SharedWorld = Arc<Mutex<World>>;

/// `GET /api/v1/home` — returns all lands currently in the world, projected
/// under `data.lands`. Reads the world on every request, so any mutation made
/// by the Update schedule is immediately visible to clients.
async fn home(State(world): State<SharedWorld>) -> Json<Value> {
    // Acquire the lock, build the response payload, then drop the guard
    // before returning so we never hold the lock across an `.await`.
    let lands: Vec<Value> = {
        let w = world.lock().expect("world mutex poisoned");
        let mut lands = Vec::new();
        for (id, name, terrain, holding, borders) in w
            .query::<(&StringId, &LandName, &LandTerrain, &LandHolding, &LandBorders)>()
            .iter()
        {
            lands.push(json!({
                "id": id.0,
                "name": name.0,
                "terrain": terrain.0,
                "holding": [holding.0, holding.1],
                "borders": borders.0,
            }));
        }
        lands
    };

    let roads: Vec<Value> = {
        let w = world.lock().expect("world mutex poisoned");
        let mut roads = Vec::new();
        for (id, points, between, days) in w
            .query::<(&StringId, &RoadPoints, &RoadBetween, &RoadDistanceDays)>()
            .iter()
        {
            roads.push(json!({
                "id": id.0,
                "points": points.0,
                "between_land_ids": between.0,
                "distance_days": days.0,
            }));
        }
        roads
    };

    let settlements: Vec<Value> = {
        let w = world.lock().expect("world mutex poisoned");
        let mut settlements = Vec::new();
        for (id, land_id, population) in w
            .query::<(&StringId, &SettlementLandId, &SettlementPopulation)>()
            .iter()
        {
            settlements.push(json!({
                "id": id.0,
                "land_id": land_id.0,
                "population": population.0,
            }));
        }
        settlements
    };

    Json(json!({ "data": { "lands": lands, "roads": roads, "settlements": settlements } }))
}

/// Build the full router: `/api/v1/*` → axum routes; everything else →
/// static files from `static_dir` with `index.html` fallback for SPA routing.
pub fn router(world: SharedWorld, static_dir: &'static str) -> Router {
    let api = Router::new()
        .route("/api/v1/home", get(home))
        .with_state(world);

    let index = format!("{static_dir}/index.html");
    let serve_dir = ServeDir::new(static_dir).fallback(ServeFile::new(index));

    api.fallback_service(serve_dir)
}

/// Spawn the HTTP server on its own OS thread with its own tokio runtime.
///
/// The main thread's loop is reserved for the Update schedule, so we keep
/// the server fully isolated. The server runs until the process exits.
pub fn serve(world: SharedWorld, addr: SocketAddr, static_dir: &'static str) {
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
                axum::serve(listener, router(world, static_dir))
                    .await
                    .expect("HTTP server crashed");
            });
        })
        .expect("failed to spawn HTTP thread");
}
