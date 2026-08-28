use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Types of terrain a land can have.
///
/// Mod files store terrain as a lowercase string (e.g. `"plains"`), so we use
/// custom `Serialize`/`Deserialize` impls to map between strings and variants
/// rather than relying on serde's default externally-tagged enum form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Plains,
    Wetlands,
    Forest,
}

impl Serialize for Terrain {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            Terrain::Plains => "plains",
            Terrain::Wetlands => "wetlands",
            Terrain::Forest => "forest",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for Terrain {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "plains" => Ok(Terrain::Plains),
            "wetlands" => Ok(Terrain::Wetlands),
            "forest" => Ok(Terrain::Forest),
            other => Err(serde::de::Error::custom(format!(
                "unknown terrain: '{}'",
                other
            ))),
        }
    }
}

/// The display name of the land (e.g., "Goldharbour").
#[derive(Debug, Clone)]
pub struct LandName(pub String);

/// The terrain type of the land.
#[derive(Debug, Clone, Copy)]
pub struct LandTerrain(pub Terrain);

/// The (x, y) position of the holding on the map.
#[derive(Debug, Clone, Copy)]
pub struct LandHolding(pub f32, pub f32);

/// Polygon vertices that form the closed border of the land.
#[derive(Debug, Clone)]
pub struct LandBorders(pub Vec<(f32, f32)>);
