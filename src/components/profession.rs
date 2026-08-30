/// One file's worth of profession records, attached to a single entity so
/// every entry in professions.ron lives in one queryable place.
#[derive(Debug, Clone)]
pub struct Professions(pub Vec<crate::content::Profession>);