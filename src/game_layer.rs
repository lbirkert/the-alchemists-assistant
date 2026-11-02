use crate::prelude::*;

#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Default, // Layer 0 - the default layer that objects are assigned to
    Player,      // Layer 1
    Npc,         // Layer 2
    Environment, // Layer 3
    Pickable,    // Layer 3
}
