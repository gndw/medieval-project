use serde::{Deserialize, Serialize};

/// A specific date in the game's calendar: `year`, `month`, `day`.
///
/// `month` is 1-indexed (1 = first month of the year) and `day` is 1-indexed
/// within a month. Validating a date against the `Calendar` component is the
/// responsibility of the game logic.
///
/// `Default` is `{ year: 0, month: 0, day: 0 }`. A `year` of `0` is treated
/// as the "uninitialised" sentinel in `content::load` and `content::startup`,
/// so values from non-`config.ron` files don't overwrite a real starting date.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}
