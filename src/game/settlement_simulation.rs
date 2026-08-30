use crate::app::{SharedApp, SharedSchedules, SharedWorld};
use crate::components::settlement::SettlementInventories;

/// `on_day` handler: log settlement count as a stand-in for the real
/// per-day settlement simulation. Replace the body when the rules land.
pub fn on_day(_app: SharedApp, world: SharedWorld, _schedules: SharedSchedules) {
    let world = world.lock().expect("world mutex poisoned");
    let count = world.query::<&SettlementInventories>().iter().count();
    println!("[on_day] settlement_simulation: {} settlements", count);
}