use serde::Serialize;
use tauri::Emitter;

use crate::app::{SharedApp, SharedWorld};
use crate::components::calendar::Calendar;
use crate::components::date::Date;

/// Payload pushed to the webview every tick. Matches `DateResponse` in
/// `web/src/lib/types.ts`.
#[derive(Debug, Clone, Serialize)]
pub struct DatePayload {
    pub date: Option<Date>,
    pub is_paused: bool,
}

/// Advance the singleton `Date` by one day, wrapping per `Calendar`.
/// Fires `on_day` on rollover and emits `date-updated` to the webview.
pub fn tick(app: SharedApp, world: SharedWorld) {
    let advanced = {
        let world = world.lock().expect("world mutex poisoned");

        // Read the calendar first (immutable borrow) so its `QueryBorrow`
        // is dropped before we open the mutable borrow on `Date` below.
        let Some(calendar) = world.query::<&Calendar>().iter().next().copied() else {
            return;
        };

        // Singleton `Date`: take the first match and mutate in place.
        let mut dates = world.query::<&mut Date>();
        if let Some(date) = dates.iter().next() {
            date.day += 1;
            if date.day > calendar.days_per_month {
                date.day = 1;
                date.month += 1;
                if date.month > calendar.months_per_year {
                    date.month = 1;
                    date.year += 1;
                }
            }
            true
        } else {
            false
        }
    };

    if advanced {
        crate::game::workplace_producing::on_day(app.clone(), world.clone());
    }

    // Emit the post-tick date to the webview. Failures are swallowed: the
    // webview may have been closed mid-flight.
    if let Some(handle) = app.tauri() {
        let date = {
            let w = world.lock().expect("world mutex poisoned");
            w.query::<&Date>().iter().next().copied()
        };
        let _ = handle.emit(
            "date-updated",
            DatePayload {
                date,
                is_paused: app.is_paused(),
            },
        );
    }
}
