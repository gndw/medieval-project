use std::fs;
use std::path::Path;

use hecs::World;
use serde::{Deserialize, Serialize};

use crate::components::core::StringId;
use crate::components::land::{LandBorders, LandHolding, LandName, LandTerrain, Terrain};

/// A single land as defined in a content file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Land {
    pub id: String,
    pub name: String,
    pub terrain: Terrain,
    pub holding: (f32, f32),
    pub borders: Vec<(f32, f32)>,
}

/// The master content structure. Loaded from `contents/base/lands.ron`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Content {
    pub lands: Vec<Land>,
}

const CONTENT_PATH: &str = "contents/base/lands.ron";

/// Load all content from disk and return the deserialised `Content` value.
pub fn load() -> Content {
    let path = Path::new(CONTENT_PATH);
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    ron::from_str::<Content>(&contents)
        .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e))
}

impl Content {
    /// Spawn one entity per land into the given world.
    pub fn Startup(&mut self, world: &mut World) {
        for land in self.lands.drain(..) {
            let (hx, hy) = land.holding;
            world.spawn((
                StringId(land.id),
                LandName(land.name),
                LandTerrain(land.terrain),
                LandHolding(hx, hy),
                LandBorders(land.borders),
            ));
        }
    }
}
