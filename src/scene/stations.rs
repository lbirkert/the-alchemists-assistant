use crate::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(EnvironmentObject, Collider::cuboid(2., 0.5, 0.8))]
struct AlchemyTable;
