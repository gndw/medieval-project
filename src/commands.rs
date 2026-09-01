use serde::Deserialize;
use serde_json::{json, Value};
use tauri::State;

use crate::app::{SharedApp, SharedWorld};
use crate::components::core::StringId;
use crate::components::date::Date;
use crate::components::inventory::{InventoryQuantity, InventoryResourceId};
use crate::components::land::{LandBorders, LandHolding, LandName, LandTerrain};
use crate::components::population::PopulationProfessionId;
use crate::components::road::{RoadBetween, RoadDistanceDays, RoadPoints};
use crate::components::settlement::{
    SettlementInventories, SettlementLandId, SettlementPopulations, SettlementWorkplaces,
};
use crate::components::workplace::{
    WorkplacePopulationId, WorkplaceProduceNextDate, WorkplaceProductionId,
};

/// Body of `set_pause` command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PausePayload {
    pub is_paused: bool,
}

/// `cmd_home` — full snapshot of lands, roads, and settlements.
/// Re-reads the world on every call so mutations are immediately visible.
#[tauri::command]
pub fn home(world: State<SharedWorld>) -> Value {
    // Acquire the lock, build the response payload, then drop the guard
    // before returning so we never hold the lock across an `.await`.
    let lands: Vec<Value> = {
        let w = world.lock().expect("world mutex poisoned");
        let mut lands = Vec::new();
        for (id, name, terrain, holding, borders) in w
            .query::<(&StringId, &LandName, &LandTerrain, &LandHolding, &LandBorders)>()
            .iter()
        {
            lands.push(json!({
                "id": id.0,
                "name": name.0,
                "terrain": terrain.0,
                "holding": [holding.0, holding.1],
                "borders": borders.0,
            }));
        }
        lands
    };

    let roads: Vec<Value> = {
        let w = world.lock().expect("world mutex poisoned");
        let mut roads = Vec::new();
        for (id, points, between, days) in w
            .query::<(&StringId, &RoadPoints, &RoadBetween, &RoadDistanceDays)>()
            .iter()
        {
            roads.push(json!({
                "id": id.0,
                "points": points.0,
                "between_land_ids": between.0,
                "distance_days": days.0,
            }));
        }
        roads
    };

    let settlements: Vec<Value> = {
        let w = world.lock().expect("world mutex poisoned");
        let mut settlements = Vec::new();

        // Collect owned copies of the per-settlement data inside a scope so the
        // `QueryBorrow` is released before we do per-child `w.get()` lookups.
        struct Row {
            id: String,
            land_id: String,
            inventories: Vec<hecs::Entity>,
            populations: Vec<hecs::Entity>,
            workplaces: Vec<hecs::Entity>,
        }
        let rows: Vec<Row> = {
            let mut q = w.query::<(
                &StringId,
                &SettlementLandId,
                &SettlementInventories,
                &SettlementPopulations,
                &SettlementWorkplaces,
            )>();
            q.iter()
                .map(|(sid, land_id, inv_ents, pop_ents, work_ents)| Row {
                    id: sid.0.clone(),
                    land_id: land_id.0.clone(),
                    inventories: inv_ents.0.clone(),
                    populations: pop_ents.0.clone(),
                    workplaces: work_ents.0.clone(),
                })
                .collect()
        };

        for row in &rows {
            let mut inventories = Vec::new();
            for &e in &row.inventories {
                let inv_id = w.get::<&StringId>(e).unwrap().0.clone();
                let res_id = w.get::<&InventoryResourceId>(e).unwrap().0.clone();
                let qty = w.get::<&InventoryQuantity>(e).unwrap().0;
                inventories.push(json!({
                    "id": inv_id,
                    "resource_id": res_id,
                    "quantity": qty,
                }));
            }

            let mut populations = Vec::new();
            for &e in &row.populations {
                let pop_id = w.get::<&StringId>(e).unwrap().0.clone();
                let prof_id = w.get::<&PopulationProfessionId>(e).unwrap().0.clone();
                populations.push(json!({
                    "id": pop_id,
                    "profession_id": prof_id,
                }));
            }

            let mut workplaces = Vec::new();
            for &e in &row.workplaces {
                let work_id = w.get::<&StringId>(e).unwrap().0.clone();
                let prod_id = w.get::<&WorkplaceProductionId>(e).unwrap().0.clone();
                let pop_id = w.get::<&WorkplacePopulationId>(e).unwrap().0.clone();
                let next_date = w.get::<&WorkplaceProduceNextDate>(e).unwrap().0;
                workplaces.push(json!({
                    "id": work_id,
                    "production_id": prod_id,
                    "population_id": pop_id,
                    "produce_next_date": next_date.map(|d| json!({
                        "year": d.year, "month": d.month, "day": d.day,
                    })),
                }));
            }

            settlements.push(json!({
                "id": row.id,
                "land_id": row.land_id,
                "inventories": inventories,
                "populations": populations,
                "workplaces": workplaces,
            }));
        }
        settlements
    };

    json!({ "data": { "lands": lands, "roads": roads, "settlements": settlements } })
}

/// `cmd_get_date` — current in-game date and pause flag.
/// `data.date` is `null` when no `Date` entity exists yet.
#[tauri::command]
pub fn get_date(world: State<SharedWorld>, app: State<SharedApp>) -> Value {
    // Copy the date out and drop the guard before returning so the lock
    // is never held across an `.await`.
    let date: Option<Date> = {
        let w = world.lock().expect("world mutex poisoned");
        w.query::<&Date>().iter().next().copied()
    };

    let date = match date {
        Some(d) => json!({ "year": d.year, "month": d.month, "day": d.day }),
        None => Value::Null,
    };

    json!({
        "data": {
            "date": date,
            "is_paused": app.is_paused(),
        }
    })
}

/// `cmd_set_pause` — pauses or resumes the tick loop, returns the applied value.
#[tauri::command]
pub fn set_pause(app: State<SharedApp>, payload: PausePayload) -> Value {
    app.set_pause(payload.is_paused);
    json!({ "data": { "is_paused": payload.is_paused } })
}
