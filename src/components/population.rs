/// The settlement entity this population member belongs to.
#[derive(Debug, Clone, Copy)]
pub struct PopulationSettlement(pub hecs::Entity);

/// String ID of the profession this population member practises.
#[derive(Debug, Clone)]
pub struct PopulationProfessionId(pub String);