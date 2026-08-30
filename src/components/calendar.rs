use serde::{Deserialize, Serialize};

/// Defines the structure of the game's calendar: every month has the same
/// length in days, and every year has the same number of months.
///
/// `Default` is `{ days_per_month: 0, months_per_year: 0 }`. Both fields at
/// `0` is the "uninitialised" sentinel in `content::load` and
/// `content::startup`, so values from non-`config.ron` files don't overwrite
/// a real calendar definition.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub days_per_month: u32,
    pub months_per_year: u32,
}
