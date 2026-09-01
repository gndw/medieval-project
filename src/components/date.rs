use serde::{Deserialize, Serialize};

/// A specific date in the game's calendar: `year`, `month` (1-indexed), `day` (1-indexed).
/// Default `year: 0` is the uninitialised sentinel in `content::load`/`startup`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}
