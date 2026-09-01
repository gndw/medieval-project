/// The settlement entity this workplace operates in.
#[derive(Debug, Clone, Copy)]
pub struct WorkplaceSettlement(pub hecs::Entity);

/// String ID of the production site template this workplace instantiates.
#[derive(Debug, Clone)]
pub struct WorkplaceProductionId(pub String);

/// String ID of the population member assigned to work this site.
#[derive(Debug, Clone)]
pub struct WorkplacePopulationId(pub String);

/// When this workplace next finishes its production cycle. `None` until the
/// workplace has been started by a matching profession; the on_day handler
/// resets it to `today + produce_cycle_days` after each harvest.
#[derive(Debug, Clone, Copy)]
pub struct WorkplaceProduceNextDate(pub Option<crate::components::date::Date>);