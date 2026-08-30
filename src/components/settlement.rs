/// The string ID of the land this settlement belongs to.
#[derive(Debug, Clone)]
pub struct SettlementLandId(pub String);

/// Entity handles of every inventory slot owned by this settlement.
#[derive(Debug, Clone)]
pub struct SettlementInventories(pub Vec<hecs::Entity>);

/// Entity handles of every population member living in this settlement.
#[derive(Debug, Clone)]
pub struct SettlementPopulations(pub Vec<hecs::Entity>);

/// Entity handles of every workplace running in this settlement.
#[derive(Debug, Clone)]
pub struct SettlementWorkplaces(pub Vec<hecs::Entity>);
