/// The ordered list of (x, y) waypoints that form the road's path.
#[derive(Debug, Clone)]
pub struct RoadPoints(pub Vec<(f32, f32)>);

/// The string IDs of the lands the road connects (typically two).
#[derive(Debug, Clone)]
pub struct RoadBetween(pub Vec<String>);

/// Travel time for the road, in in-game days.
#[derive(Debug, Clone, Copy)]
pub struct RoadDistanceDays(pub u32);
