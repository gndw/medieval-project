use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::app::SharedWorld;
use crate::components::core::StringId;
use crate::components::land::{LandBorders, LandHolding, LandName, LandTerrain};
use crate::components::road::{RoadBetween, RoadDistanceDays, RoadPoints};
use crate::components::settlement::{SettlementLandId, SettlementPopulation};

/// `GET /api/v1/home` — returns all lands, roads, and settlements in `data`.
/// Re-reads the world on every request so mutations are immediately visible.
pub async fn home(State(world): State<SharedWorld>) -> Json<Value> {
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
        for (id, land_id, population) in w
            .query::<(&StringId, &SettlementLandId, &SettlementPopulation)>()
            .iter()
        {
            settlements.push(json!({
                "id": id.0,
                "land_id": land_id.0,
                "population": population.0,
            }));
        }
        settlements
    };

    Json(json!({ "data": { "lands": lands, "roads": roads, "settlements": settlements } }))
}
