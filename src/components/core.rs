/// A unique string identifier attached to an entity, used to look it up by ID
/// (e.g. resolving `LandBorders` neighbour lists against world entities).
#[derive(Debug, Clone)]
pub struct StringId(pub String);
