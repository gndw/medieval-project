mod components;

use components::{Name, Position, Velocity};
use hecs::*;

fn main() {
    let mut world = World::new();

    // Spawn entities with components
    world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0, dy: 0.5 }));
    world.spawn((Position { x: 5.0, y: 5.0 }, Name("Knight".to_string())));
    world.spawn((
        Position { x: -3.0, y: 2.0 },
        Velocity { dx: 0.0, dy: -1.0 },
        Name("Archer".to_string()),
    ));

    println!("=== Initial State ===");
    // Query yields just the components that match
    for (pos, name) in world.query::<(&Position, &Name)>().iter() {
        println!("{} at ({}, {})", name.0, pos.x, pos.y);
    }

    // System: integrate velocity into position
    for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }

    println!("\n=== After Update ===");
    for pos in world.query::<&Position>().iter() {
        println!("Position: ({}, {})", pos.x, pos.y);
    }

    println!("\nTotal entities: {}", world.iter().count());
}
