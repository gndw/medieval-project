use std::fs;
use std::path::Path;

use hecs::World;
use serde::{Deserialize, Serialize};

use crate::components::core::StringId;
use crate::components::land::{LandBorders, LandHolding, LandName, LandTerrain, Terrain};
use crate::components::road::{RoadBetween, RoadDistanceDays, RoadPoints};
use crate::components::settlement::{SettlementLandId, SettlementPopulation};

/// A single land as defined in a content file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Land {
    pub id: String,
    pub name: String,
    pub terrain: Terrain,
    pub holding: (f32, f32),
    pub borders: Vec<(f32, f32)>,
}

/// A single road as defined in a content file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    pub id: String,
    pub points: Vec<(f32, f32)>,
    pub between_land_ids: Vec<String>,
    pub distance_days: u32,
}

/// A single settlement as defined in a content file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub id: String,
    pub land_id: String,
    pub population: u32,
}

/// The master content structure. Loaded from `contents/base/lands.ron`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Content {
    pub lands: Vec<Land>,
}

const CONTENT_PATH: &str = "contents/base/lands.ron";
const ROADS_PATH: &str = "contents/base/roads.ron";
const SETTLEMENTS_PATH: &str = "contents/base/settlements.ron";

/// Load all land content from disk and return the deserialised `Content` value.
pub fn load() -> Content {
    let path = Path::new(CONTENT_PATH);
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    ron::from_str::<Content>(&contents)
        .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e))
}

/// Load road content from disk and return the deserialised road list.
pub fn load_roads() -> Vec<Road> {
    let path = Path::new(ROADS_PATH);
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    ron::from_str::<RoadsFile>(&contents)
        .map(|f| f.roads)
        .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e))
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RoadsFile {
    roads: Vec<Road>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SettlementsFile {
    settlements: Vec<Settlement>,
}

/// Load settlement content from disk and return the deserialised list.
pub fn load_settlements() -> Vec<Settlement> {
    let path = Path::new(SETTLEMENTS_PATH);
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    ron::from_str::<SettlementsFile>(&contents)
        .map(|f| f.settlements)
        .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e))
}

/// Load land content from disk and spawn one entity per land into `world`.
pub fn startup(world: &mut World) {
    let mut content = load();
    for land in content.lands.drain(..) {
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

/// Load road content from disk and spawn one entity per road into `world`.
pub fn roads_startup(world: &mut World) {
    for road in load_roads() {
        world.spawn((
            StringId(road.id),
            RoadPoints(road.points),
            RoadBetween(road.between_land_ids),
            RoadDistanceDays(road.distance_days),
        ));
    }
}

/// Load settlement content from disk and spawn one entity per settlement into `world`.
pub fn settlements_startup(world: &mut World) {
    for s in load_settlements() {
        world.spawn((
            StringId(s.id),
            SettlementLandId(s.land_id),
            SettlementPopulation(s.population),
        ));
    }
}
