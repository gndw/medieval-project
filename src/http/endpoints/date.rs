use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::app::SharedWorld;
use crate::components::date::Date;

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
