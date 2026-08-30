use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::app::{SharedApp, SharedWorld};
use crate::components::date::Date;

/// Body of `POST /api/v1/date`.
#[derive(Debug, Deserialize)]
pub struct PausePayload {
    pub is_paused: bool,
}

/// `GET /api/v1/date` — returns the game date and the pause flag.
/// `data.date` is `null` when no `Date` entity exists yet.
pub async fn date(
    State(world): State<SharedWorld>,
    State(app): State<SharedApp>,
) -> Json<Value> {
    // Copy the date out and drop the guard before returning so the lock
    // is never held across an `.await`.
    let date: Option<Date> = {
        let w = world.lock().expect("world mutex poisoned");
        w.query::<&Date>().iter().next().copied()
    };

    let date = match date {
        Some(d) => json!({ "year": d.year, "month": d.month, "day": d.day }),
        None => Value::Null,
    };

    Json(json!({
        "data": {
            "date": date,
            "is_paused": app.is_paused(),
        }
    }))
}

/// `POST /api/v1/date` — pauses or resumes the tick loop via
/// `AppState::set_pause`, then echoes the applied value.
pub async fn set_pause(
    State(app): State<SharedApp>,
    Json(payload): Json<PausePayload>,
) -> Json<Value> {
    app.set_pause(payload.is_paused);
    Json(json!({ "data": { "is_paused": payload.is_paused } }))
}
