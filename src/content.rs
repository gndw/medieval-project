use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::app::SharedWorld;
use crate::components::calendar::Calendar;
use crate::components::core::StringId;
use crate::components::date::Date;
use crate::components::land::{LandBorders, LandHolding, LandName, LandTerrain, Terrain};
use crate::components::road::{RoadBetween, RoadDistanceDays, RoadPoints};
use crate::components::settlement::{SettlementLandId, SettlementPopulation};

const CONTENT_DIR: &str = "contents/base";

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

/// Aggregated content loaded from every `.ron` file under `contents/base/`.
/// Each file may declare any subset of fields; absent ones fall back to `Default`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Content {
    pub lands: Vec<Land>,
    pub roads: Vec<Road>,
    pub settlements: Vec<Settlement>,
    pub calendar: Calendar,
    pub date: Date,
}

/// Load all `.ron` content from `contents/base/` and merge into one `Content`.
/// Files are read in sorted order; vectors concat, scalars use first-non-default.
pub fn load() -> Content {
    let dir = Path::new(CONTENT_DIR);
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read directory {}: {}", dir.display(), e));

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().map(|ext| ext == "ron").unwrap_or(false)
        })
        .collect();
    paths.sort();

    let mut content = Content::default();
    for path in &paths {
        let raw = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
        let parsed = ron::from_str::<Content>(&raw)
            .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e));
        content.lands.extend(parsed.lands);
        content.roads.extend(parsed.roads);
        content.settlements.extend(parsed.settlements);
        if content.calendar == Calendar::default() {
            content.calendar = parsed.calendar;
        }
        if content.date == Date::default() {
            content.date = parsed.date;
        }
    }

    // Require a starting `date` from config.ron; serde(default) would
    // otherwise silently fall back to zeros and leave the game with no origin.
    if content.date == Date::default() {
        panic!("contents/base/config.ron must define a `date` entry");
    }

    content
}

/// Load content and spawn lands, roads, settlements, and the time-system
/// resource entities (`Calendar` and `Date`) into `world`.
pub fn startup(world: SharedWorld) {
    let content = load();

    let mut world = world.lock().expect("world mutex poisoned");

    for land in content.lands {
        let (hx, hy) = land.holding;
        world.spawn((
            StringId(land.id),
            LandName(land.name),
            LandTerrain(land.terrain),
            LandHolding(hx, hy),
            LandBorders(land.borders),
        ));
    }

    for road in content.roads {
        world.spawn((
            StringId(road.id),
            RoadPoints(road.points),
            RoadBetween(road.between_land_ids),
            RoadDistanceDays(road.distance_days),
        ));
    }

    for s in content.settlements {
        world.spawn((
            StringId(s.id),
            SettlementLandId(s.land_id),
            SettlementPopulation(s.population),
        ));
    }

    // Spawn singleton resource entities for the time system, one per
    // component, so game logic can query for `&Date` or `&Calendar` independently.
    world.spawn((content.calendar,));
    if content.date != Date::default() {
        world.spawn((content.date,));
    }
}
