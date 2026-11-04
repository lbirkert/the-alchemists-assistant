use bevy::picking::backend::HitData;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_message::<PlayerInteraction>();

    app.add_systems(GameUpdate, interaction);
}

#[derive(Message, Debug)]
pub struct PlayerInteraction {
    pub entity: Entity,
    pub kind: Interactable,
    pub hit: HitData,
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub enum Interactable {
    Item(Item),
    PlaceableLocation,
    Discardable,
    Source { item: Item },
    Drawer,
    Mortar,
}

fn interaction(
    input: Res<ButtonInput<MouseButton>>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    query: Query<(Option<&ChildOf>, Has<Interactable>)>,
    interactables: Query<&Interactable>,
    mut messages: MessageWriter<PlayerInteraction>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if let Some((mesh_entity, hit)) = pointer.get_nearest_hit()
        && let Some(entity) = crate::utils::find(*mesh_entity, &query)
        && let Ok(interactable) = interactables.get(entity)
    {
        messages.write(PlayerInteraction {
            entity,
            kind: *interactable,
            hit: hit.clone(),
        });
    }
}
