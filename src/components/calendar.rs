use serde::{Deserialize, Serialize};

/// Defines the game's calendar: every month has `days_per_month` days,
/// every year has `months_per_year` months. Default of zero is the uninit sentinel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub days_per_month: u32,
    pub months_per_year: u32,
}
