use std::sync::atomic::Ordering;

use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::app::{SharedPaused, SharedWorld};
use crate::components::date::Date;

/// Body of `POST /api/v1/date`.
#[derive(Debug, Deserialize)]
pub struct PausePayload {
    pub is_paused: bool,
}

/// `GET /api/v1/date` — returns the singleton game date in `data`.
/// `data` is `null` when no `Date` entity exists yet.
pub async fn date(State(world): State<SharedWorld>) -> Json<Value> {
    // Copy the date out and drop the guard before returning so the lock
    // is never held across an `.await`.
    let date: Option<Date> = {
        let w = world.lock().expect("world mutex poisoned");
        w.query::<&Date>().iter().next().copied()
    };

    let data = match date {
        Some(d) => json!({ "year": d.year, "month": d.month, "day": d.day }),
        None => Value::Null,
    };

    Json(json!({ "data": data }))
}

/// `POST /api/v1/date` — sets `App::is_tick_paused` from `is_paused`.
/// While paused the main loop skips every registered tick.
pub async fn set_pause(
    State(is_tick_paused): State<SharedPaused>,
    Json(payload): Json<PausePayload>,
) -> Json<Value> {
    is_tick_paused.store(payload.is_paused, Ordering::Relaxed);
    Json(json!({ "data": { "is_paused": payload.is_paused } }))
}
