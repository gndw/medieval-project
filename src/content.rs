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
///
/// Each file may declare any subset of `lands`, `roads`, `settlements`,
/// `calendar`, or `date`; fields that are absent from a file fall back to
/// their `Default` values. `config.ron` is the only file that supplies the
/// time-system fields (`calendar` and `date`); the entity files
/// (`lands.ron`, `roads.ron`, `settlements.ron`) parse with their default
/// (zero) values for those fields, which the merge logic then ignores.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Content {
    pub lands: Vec<Land>,
    pub roads: Vec<Road>,
    pub settlements: Vec<Settlement>,
    pub calendar: Calendar,
    pub date: Date,
}

/// Load all content from disk and return the deserialised `Content` value.
///
/// Reads every `.ron` file under `contents/base/`, parses each into a
/// `Content`, and merges the results into a single `Content`. Files are
/// processed in sorted order so repeated loads are deterministic. Vector
/// fields are concatenated; `calendar` and `date` use a "first non-default
/// wins" rule, so `config.ron` (which sorts first) supplies those values
/// and the zero defaults from the entity files are ignored.
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

    // `config.ron` is required to supply a starting `date`; otherwise the
    // game has no time origin. We validate here (rather than relying on RON
    // to reject a missing file) because `#[serde(default)]` on `Content`
    // lets files quietly fall back to zero defaults.
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

    // Spawn singleton resource entities for the time system. Each component
    // is attached to its own entity so game logic can query for `&Date` or
    // `&Calendar` independently. `Date` is required to be present in
    // `config.ron` (`load` panics otherwise); the runtime check here is
    // kept defensively. `Calendar` falls back to its `Default` if absent.
    world.spawn((content.calendar,));
    if content.date != Date::default() {
        world.spawn((content.date,));
    }
}
