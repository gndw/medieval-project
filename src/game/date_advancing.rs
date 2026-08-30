use crate::app::SharedWorld;
use crate::components::calendar::Calendar;
use crate::components::date::Date;

/// Advance the in-game date by one day. Pulls the singleton `Calendar` and
/// `Date` entities from the world and rolls the date forward, wrapping at
/// month/year boundaries according to the calendar definition.
///
/// If no `Calendar` entity exists, this tick is a no-op (there is nothing
/// to advance against).
pub fn tick(world: SharedWorld) {
    let world = world.lock().expect("world mutex poisoned");

    // Read the calendar first (immutable borrow) so we know the month/year
    // lengths. The `QueryBorrow` returned by `query` is dropped at the end
    // of this statement, releasing the immutable borrow before we open the
    // mutable borrow below.
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
