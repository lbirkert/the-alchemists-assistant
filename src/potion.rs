use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(GamePostUpdate, set_potion_liquid_material);
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
#[component(on_remove)]
pub struct PotionLiquid {
    material_handle: Option<Handle<StandardMaterial>>,
}

impl PotionLiquid {
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        let Some(potion_liquid) = world.get::<PotionLiquid>(context.entity).cloned() else {
            return;
        };

        let mut commands = world.commands();

        if let Some(material) = potion_liquid.material_handle {
            commands.queue(move |world: &mut World| {
                world
                    .resource_mut::<Assets<StandardMaterial>>()
                    .remove(material.id());
            });
        }
    }
}

fn set_potion_liquid_material(
    mut commands: Commands,
    mut potion_liquids: Query<(Entity, &mut PotionLiquid)>,
    items: Query<&Item>,
    hierarchy: Query<(Option<&ChildOf>, Has<Item>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut potion_liquid) in &mut potion_liquids {
        if potion_liquid.material_handle.is_some() {
            continue;
        }

        let Some(item_entity) = crate::utils::find_parent::<Item>(entity, &hierarchy) else {
            continue;
        };

        let Ok(item) = items.get(item_entity) else {
            continue;
        };

        let Some(potion) = &item.potion else {
            continue;
        };

        if potion.is_empty() {
            commands.queue(move |world: &mut World| {
                if let Ok(mut entity) = world.get_entity_mut(entity) {
                    entity.insert(Visibility::Hidden);
                }
            });
            continue;
        }

        let material_handle = materials.add(StandardMaterial {
            base_color: potion.color(),
            ..default()
        });

        potion_liquid.material_handle = Some(material_handle.clone());

        commands
            .entity(entity)
            .insert(MeshMaterial3d(material_handle));
    }
}

// a potion crafted by a user that might have a sticky note attached
#[derive(Reflect, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[reflect(Default)]
pub struct Potion {
    items: Vec<Item>,
    // TODO: type: round, square, etc...
    sticky_note: Option<StickyNote>,
}

impl Potion {
    pub fn can_validate(&self) -> bool {
        self.sticky_note
            .as_ref()
            .is_some_and(|s| s.recipe.is_some())
    }

    pub fn validate(&self) -> Option<bool> {
        if let Some(StickyNote {
            recipe: Some(ref recipe),
            ..
        }) = self.sticky_note
        {
            Some(self.items == recipe.items())
        } else {
            None
        }
    }

    pub fn color(&self) -> Color {
        Color::srgb(1.0, 0.0, 0.0)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
