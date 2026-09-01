use std::collections::HashMap;

use crate::app::{SharedApp, SharedSchedules, SharedWorld};
use crate::components::calendar::Calendar;
use crate::components::core::StringId;
use crate::components::date::Date;
use crate::components::inventory::{InventoryQuantity, InventoryResourceId, InventorySettlement};
use crate::components::population::PopulationProfessionId;
use crate::components::production::Productions;
use crate::components::workplace::{
    WorkplacePopulationId, WorkplaceProduceNextDate, WorkplaceProductionId, WorkplaceSettlement,
};

/// Advance each workplace's production cycle by one day.
///
/// Two cases per workplace:
///   1. Has a next-date and it has arrived → harvest `produces`, reschedule.
///   2. Has no next-date and the assigned population's profession matches the
///      production's required profession → consume `consumes`, schedule harvest.
pub fn on_day(_app: SharedApp, world: SharedWorld, _schedules: SharedSchedules) {
    let mut world = world.lock().expect("world mutex poisoned");

    let today = match world.query::<&Date>().iter().next().copied() {
        Some(d) => d,
        None => return,
    };
    let calendar = world
        .query::<&Calendar>()
        .iter()
        .next()
        .copied()
        .unwrap_or_default();

    // Clone the production table out so the QueryBorrow drops before mutations.
    let productions: Vec<crate::content::Production> = {
        let mut q = world.query::<&Productions>();
        match q.iter().next() {
            Some(p) => p.0.clone(),
            None => return,
        }
    };

    // Snapshot every workplace's identifying fields so the QueryBorrow drops.
    struct WpSnap {
        entity: hecs::Entity,
        settlement: hecs::Entity,
        production_id: String,
        population_id: String,
        next_date: Option<Date>,
    }
    let workplaces: Vec<WpSnap> = {
        let mut q = world.query::<(
            hecs::Entity,
            &WorkplaceSettlement,
            &WorkplaceProductionId,
            &WorkplacePopulationId,
            &WorkplaceProduceNextDate,
        )>();
        q.iter()
            .map(|(e, ws, wp, wpop, wnd)| WpSnap {
                entity: e,
                settlement: ws.0,
                production_id: wp.0.clone(),
                population_id: wpop.0.clone(),
                next_date: wnd.0,
            })
            .collect()
    };

    // Map population StringId → profession_id.
    let population_profession: HashMap<String, String> = {
        let mut q = world.query::<(&StringId, &PopulationProfessionId)>();
        q.iter().map(|(id, p)| (id.0.clone(), p.0.clone())).collect()
    };

    // Map (settlement, resource_id) → inventory entity would be built here
    // for O(1) lookup, but each workplace touches only a handful of resources
    // per tick, so a per-call linear scan via `find_inventory` is cheaper
    // than materialising the full index.

    for wp in &workplaces {
        let Some(production) = productions.iter().find(|p| p.id == wp.production_id) else {
            continue;
        };

        match wp.next_date {
            Some(d) if d <= today => {
                // Cycle complete: harvest produces and clear the next-date so
                // the next tick can re-enter the `None` branch and consume
                // inputs for the following cycle.
                harvest(production, wp.settlement, &mut world);
                if let Ok(mut wnd) = world.get::<&mut WorkplaceProduceNextDate>(wp.entity) {
                    wnd.0 = None;
                }
            }
            Some(_) => {
                // Cycle is in progress; not yet time to harvest.
                continue;
            }
            None => {
                let Some(pop_prof) = population_profession.get(&wp.population_id) else {
                    continue;
                };
                if pop_prof != &production.profession_id {
                    continue;
                }
                if !can_consume(production, wp.settlement, &world) {
                    continue;
                }
                consume(production, wp.settlement, &mut world);
                let next_d = add_days(today, production.produce_cycle_days, calendar);
                if let Ok(mut wnd) = world.get::<&mut WorkplaceProduceNextDate>(wp.entity) {
                    wnd.0 = Some(next_d);
                }
            }
        }
    }
}

fn add_days(date: Date, days: u32, calendar: Calendar) -> Date {
    let mut d = date;
    for _ in 0..days {
        d.day += 1;
        if d.day > calendar.days_per_month {
            d.day = 1;
            d.month += 1;
            if d.month > calendar.months_per_year {
                d.month = 1;
                d.year += 1;
            }
        }
    }
    d
}

fn harvest(production: &crate::content::Production, settlement: hecs::Entity, world: &mut hecs::World) {
    for out in &production.produces {
        if let Some(inv_e) = find_inventory(settlement, &out.id, world) {
            if let Ok(mut q) = world.get::<&mut InventoryQuantity>(inv_e) {
                q.0 = q.0.saturating_add(out.quantity);
            }
        }
    }
}

fn can_consume(production: &crate::content::Production, settlement: hecs::Entity, world: &hecs::World) -> bool {
    production.consumes.iter().all(|con| {
        find_inventory(settlement, &con.id, world).is_some_and(|inv_e| {
            world
                .get::<&InventoryQuantity>(inv_e)
                .is_ok_and(|q| q.0 >= con.quantity)
        })
    })
}

fn consume(production: &crate::content::Production, settlement: hecs::Entity, world: &mut hecs::World) {
    for con in &production.consumes {
        if let Some(inv_e) = find_inventory(settlement, &con.id, world) {
            if let Ok(mut q) = world.get::<&mut InventoryQuantity>(inv_e) {
                q.0 = q.0.saturating_sub(con.quantity);
            }
        }
    }
}

fn find_inventory(settlement: hecs::Entity, resource_id: &str, world: &hecs::World) -> Option<hecs::Entity> {
    let mut q = world.query::<(hecs::Entity, &InventorySettlement, &InventoryResourceId)>();
    q.iter()
        .find(|(_e, s, r)| s.0 == settlement && r.0 == resource_id)
        .map(|(e, _s, _r)| e)
}