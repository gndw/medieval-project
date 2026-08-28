/// A unique string identifier attached to an entity.
///
/// Used to look up entities by ID — for example, when resolving the
/// `LandBorders` list (which contains string IDs of neighbouring lands)
/// against the world's entities.
#[derive(Debug, Clone)]
pub struct StringId(pub String);
