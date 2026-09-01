use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::app::{SharedApp, SharedWorld};
use crate::components::calendar::Calendar;
use crate::components::core::StringId;
use crate::components::date::Date;
use crate::components::inventory::{
    InventoryQuantity, InventoryResourceId, InventorySettlement,
};
use crate::components::land::{LandBorders, LandHolding, LandName, LandTerrain, Terrain};
use crate::components::population::{PopulationProfessionId, PopulationSettlement};
use crate::components::production::Productions;
use crate::components::profession::Professions;
use crate::components::resource::Resources;
use crate::components::road::{RoadBetween, RoadDistanceDays, RoadPoints};
use crate::components::settlement::{
    SettlementInventories, SettlementLandId, SettlementPopulations, SettlementWorkplaces,
};
use crate::components::workplace::{
    WorkplacePopulationId, WorkplaceProduceNextDate, WorkplaceProductionId, WorkplaceSettlement,
};

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
#[serde(default)]
pub struct Settlement {
    pub id: String,
    pub land_id: String,
    pub inventories: Vec<Inventory>,
    pub populations: Vec<Population>,
    pub workplaces: Vec<Workplace>,
}

impl Default for Settlement {
    fn default() -> Self {
        Settlement {
            id: String::new(),
            land_id: String::new(),
            inventories: Vec::new(),
            populations: Vec::new(),
            workplaces: Vec::new(),
        }
    }
}

/// One stock entry in a settlement's inventories list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub id: String,
    pub resource_id: String,
    pub quantity: u32,
}

/// One person in a settlement's populations list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Population {
    pub id: String,
    pub profession_id: String,
}

/// One production site in a settlement's workplaces list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Workplace {
    pub id: String,
    pub production_id: String,
    pub population_id: String,
    pub produce_next_date: Option<Date>,
}

impl Default for Workplace {
    fn default() -> Self {
        Workplace {
            id: String::new(),
            production_id: String::new(),
            population_id: String::new(),
            produce_next_date: None,
        }
    }
}

/// A single resource as defined in `resources.ron`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub name: String,
}

/// A single profession as defined in `professions.ron`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profession {
    pub id: String,
    pub name: String,
}

/// A reference to a resource with a quantity, used for production inputs/outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuantity {
    pub id: String,
    pub quantity: u32,
}

/// A single production as defined in `productions.ron`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Production {
    pub id: String,
    pub name: String,
    pub profession_id: String,
    pub produce_cycle_days: u32,
    pub consumes: Vec<ResourceQuantity>,
    pub produces: Vec<ResourceQuantity>,
}

impl Default for Production {
    fn default() -> Self {
        Production {
            id: String::new(),
            name: String::new(),
            profession_id: String::new(),
            produce_cycle_days: 0,
            consumes: Vec::new(),
            produces: Vec::new(),
        }
    }
}

/// Aggregated content loaded from every `.ron` file under `contents/base/`.
/// Each file may declare any subset of fields; absent ones fall back to `Default`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Content {
    pub lands: Vec<Land>,
    pub roads: Vec<Road>,
    pub settlements: Vec<Settlement>,
    pub resources: Vec<Resource>,
    pub professions: Vec<Profession>,
    pub productions: Vec<Production>,
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
        content.resources.extend(parsed.resources);
        content.professions.extend(parsed.professions);
        content.productions.extend(parsed.productions);
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
pub fn startup(_app: SharedApp, world: SharedWorld) {
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
        // Spawn the settlement first so children can hold an Entity back-ref
        // to it, then attach the aggregate child-lists after spawning them.
        let settlement = world.spawn((
            StringId(s.id.clone()),
            SettlementLandId(s.land_id),
        ));

        let mut inventory_entities = Vec::with_capacity(s.inventories.len());
        for inv in s.inventories {
            let e = world.spawn((
                StringId(inv.id),
                InventorySettlement(settlement),
                InventoryResourceId(inv.resource_id),
                InventoryQuantity(inv.quantity),
            ));
            inventory_entities.push(e);
        }

        let mut population_entities = Vec::with_capacity(s.populations.len());
        for pop in s.populations {
            let e = world.spawn((
                StringId(pop.id),
                PopulationSettlement(settlement),
                PopulationProfessionId(pop.profession_id),
            ));
            population_entities.push(e);
        }

        let mut workplace_entities = Vec::with_capacity(s.workplaces.len());
        for work in s.workplaces {
            let e = world.spawn((
                StringId(work.id),
                WorkplaceSettlement(settlement),
                WorkplaceProductionId(work.production_id),
                WorkplacePopulationId(work.population_id),
                WorkplaceProduceNextDate(work.produce_next_date),
            ));
            workplace_entities.push(e);
        }

        world
            .insert(
                settlement,
                (
                    SettlementInventories(inventory_entities),
                    SettlementPopulations(population_entities),
                    SettlementWorkplaces(workplace_entities),
                ),
            )
            .expect("settlement entity still alive");
    }

    // Each reference-data file becomes a single entity holding the whole
    // collection, so callers query one place for "all resources", etc.
    if !content.resources.is_empty() {
        world.spawn((StringId("resources".into()), Resources(content.resources)));
    }
    if !content.professions.is_empty() {
        world.spawn((StringId("professions".into()), Professions(content.professions)));
    }
    if !content.productions.is_empty() {
        world.spawn((StringId("productions".into()), Productions(content.productions)));
    }

    // Spawn singleton resource entities for the time system, one per
    // component, so game logic can query for `&Date` or `&Calendar` independently.
    world.spawn((content.calendar,));
    if content.date != Date::default() {
        world.spawn((content.date,));
    }
}
