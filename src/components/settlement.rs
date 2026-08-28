/// The string ID of the land this settlement belongs to.
#[derive(Debug, Clone)]
pub struct SettlementLandId(pub String);

/// The settlement's population.
#[derive(Debug, Clone, Copy)]
pub struct SettlementPopulation(pub u32);
