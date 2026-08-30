/// One file's worth of resource records, attached to a single entity so
/// every entry in resources.ron lives in one queryable place.
#[derive(Debug, Clone)]
pub struct Resources(pub Vec<crate::content::Resource>);