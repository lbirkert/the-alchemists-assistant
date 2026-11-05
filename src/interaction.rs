use bevy::picking::backend::HitData;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_message::<PlayerInteraction>();
    app.init_resource::<PlayerFocus>();

    app.add_systems(GameUpdate, interaction);
}

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct PlayerFocus {
    /// Stores the Entity and Interactable component of the entity currently focused by the player,
    /// if any.
    pub interactable: Option<(Entity, Interactable)>,
}

#[derive(Message, Debug)]
pub struct PlayerInteraction {
    pub entity: Entity,
    pub kind: Interactable,
    pub hit: HitData,
}

#[derive(Component, Reflect, Debug, Default, Clone, PartialEq, Eq)]
#[reflect(Component)]
#[require(DespawnOnExit<GameState> = DespawnOnExit(GameState::Game))]
pub enum Interactable {
    Item(Item),
    StickyNote(StickyNote),
    #[default]
    ItemPlaceable,
    StickyNotePlaceable,
    ItemDiscardable,
    Source {
        item: Item,
    },
    Drawer,
    Station,
}

fn interaction(
    input: Res<ButtonInput<MouseButton>>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    query: Query<(Option<&ChildOf>, Has<Interactable>)>,
    interactables: Query<&Interactable>,
    mut messages: MessageWriter<PlayerInteraction>,
    mut focus: ResMut<PlayerFocus>,
) {
    if let Some((mesh_entity, hit)) = pointer.get_nearest_hit()
        && hit.depth < 5.
        && let Some(entity) = crate::utils::find_parent(*mesh_entity, &query)
        && let Ok(interactable) = interactables.get(entity)
    {
        if Some((entity, interactable.clone())) != focus.interactable {
            focus.interactable = Some((entity, interactable.clone()));
        }
        if input.just_pressed(MouseButton::Right) {
            messages.write(PlayerInteraction {
                entity,
                kind: interactable.clone(),
                hit: hit.clone(),
            });
        }
    } else {
        focus.interactable = None;
    }
}
