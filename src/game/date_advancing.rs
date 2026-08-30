use crate::app::{SharedApp, SharedWorld};
use crate::components::calendar::Calendar;
use crate::components::date::Date;

/// Advance the singleton `Date` by one day, wrapping at month/year
/// boundaries per the `Calendar`. No-op if no `Calendar` entity exists.
pub fn tick(_app: SharedApp, world: SharedWorld) {
    let world = world.lock().expect("world mutex poisoned");

    // Read the calendar first (immutable borrow) so its `QueryBorrow` is
    // dropped before we open the mutable borrow on `Date` below.
    let Some(calendar) = world
        .query::<&Calendar>()
        .iter()
        .next()
        .copied()
    else {
        // No calendar resource — nothing to advance against.
        return;
    };

    // Now mutate the singleton Date. If the entity doesn't exist this is a
    // no-op (the for loop body never runs).
    let mut dates = world.query::<&mut Date>();
    for date in dates.iter() {
        date.day += 1;
        if date.day > calendar.days_per_month {
            date.day = 1;
            date.month += 1;
            if date.month > calendar.months_per_year {
                date.month = 1;
                date.year += 1;
            }
        }
    }
}
