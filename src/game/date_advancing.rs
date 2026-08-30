use crate::app::{SharedApp, SharedSchedules, SharedWorld};
use crate::components::calendar::Calendar;
use crate::components::date::Date;

/// `on_tick` handler: advance the singleton `Date` by one day, wrapping at
/// month/year boundaries per the `Calendar`. Fires `on_day` when a day
/// actually rolls over.
pub fn tick(app: SharedApp, world: SharedWorld, schedules: SharedSchedules) {
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
        schedules.fire("on_day", app, world);
    }
}