use bevy::picking::backend::HitData;

use crate::{item::ITEM_SIZE, prelude::*, scene::NpcTable};

pub fn plugin(app: &mut App) {
    app.add_systems(GamePostUpdate, set_potion_liquid_material);
    app.add_systems(GameUpdate, fill_potion);
    app.add_systems(GameUpdate, label_potion);
    app.add_systems(GameUpdate, submit_potion);
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
#[require(Interactable::Potion)]
pub struct Potion {
    pub data: Option<PotionData>,
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

        match item.potion {
            None => {
                commands.queue(move |world: &mut World| {
                    if let Ok(mut entity) = world.get_entity_mut(entity) {
                        entity.insert(Visibility::Hidden);
                    }
                });
            }
            Some(ref potion) => {
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
    }
}

// a potion crafted by a user that might have a sticky note attached
#[derive(Reflect, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[reflect(Default)]
pub struct PotionData {
    pub items: Vec<Item>,
    pub recipe: Option<Recipe>,
    // TODO: type: round, square, etc...
}

impl PotionData {
    pub fn can_validate(&self) -> bool {
        self.recipe.is_some()
    }

    pub fn validate(&self) -> Option<bool> {
        Some(self.items == self.recipe.as_ref()?.items())
    }

    pub fn color(&self) -> Color {
        Color::srgb(1.0, 0.0, 0.0)
    }
}

fn fill_potion(
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
    mut cauldron: Single<&mut Cauldron>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::Cauldron,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !matches!(
        inventory.selection,
        Some(Selection::Item(Item {
            kind: ItemKind::Potion,
            potion: None,
            ..
        }))
    ) {
        return;
    }

    let potion = PotionData {
        items: cauldron.items.clone(),
        recipe: None,
    };
    cauldron.items.truncate(0);
    inventory.selection.replace(Selection::Item(Item {
        kind: ItemKind::Potion,
        potion: Some(potion),
        ..Default::default()
    }));
}

fn label_potion(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        entity,
        kind: Interactable::Item(item),
        ..
    }) = messages.read().next()
    else {
        return;
    };

    let mut item = item.clone();
    let Item {
        potion: Some(ref mut potion @ PotionData { recipe: None, .. }),
        ..
    } = item
    else {
        return;
    };

    if !matches!(inventory.selection, Some(Selection::StickyNote(..))) {
        return;
    };

    let Some(Selection::StickyNote(sticky_note)) = inventory.selection.take() else {
        panic!();
    };

    potion.recipe = sticky_note.recipe.clone();

    // update potion (so that sticky note shows)
    commands
        .entity(*entity)
        .insert((Interactable::Item(item.clone()), item));
}

fn submit_potion(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
    mut npc_table: Single<&mut NpcTable>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::NpcTable,
        hit:
            HitData {
                position: Some(position),
                normal: Some(normal),
                ..
            },
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !matches!(
        inventory.selection,
        Some(Selection::Item(Item {
            potion: Some(PotionData {
                recipe: Some(..),
                ..
            }),
            ..
        }))
    ) {
        return;
    };

    let Some(Selection::Item(item)) = inventory.selection.take() else {
        panic!();
    };

    // TODO: we should maybe move the potion to a predefined grid instead of taking hit position
    let scene = item.clone().scene(
        &asset_server,
        Transform::from_translation(*position + *normal * (ITEM_SIZE / 2.0)),
    );

    let potion_entity = commands.spawn_scene(scene).id();
    npc_table
        .potions
        .push((potion_entity, item.potion.unwrap()));
}
