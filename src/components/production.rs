/// One file's worth of production records, attached to a single entity so
/// every entry in productions.ron lives in one queryable place.
#[derive(Debug, Clone)]
pub struct Productions(pub Vec<crate::content::Production>);