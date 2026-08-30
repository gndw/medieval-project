/// The settlement entity that owns this inventory slot.
#[derive(Debug, Clone, Copy)]
pub struct InventorySettlement(pub hecs::Entity);

/// String ID of the resource type stocked in this inventory slot.
#[derive(Debug, Clone)]
pub struct InventoryResourceId(pub String);

/// How many units of the resource are currently held.
#[derive(Debug, Clone, Copy)]
pub struct InventoryQuantity(pub u32);