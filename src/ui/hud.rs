use crate::{drawer::Drawer, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GamePaused(false)), setup);
    app.add_systems(GameUpdate, update_hud_text);
}

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
struct HudText;

fn setup(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #GameHud
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            align_items: AlignItems::End,
            justify_content: JustifyContent::End,
            padding: UiRect {
                right: px(20),
                bottom: px(10)
            }
        }
        Pickable::IGNORE
        DespawnOnExit::<GamePaused>(GamePaused(false))
        Children [
            HudText
            Text::new("")
        ]
    });
}

fn update_hud_text(
    inventory: Single<&Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    interactable_search: Query<(Option<&ChildOf>, Has<Interactable>)>,
    interactables: Query<&Interactable>,
    drawers: Query<&Drawer>,
    mut text: Single<&mut Text, With<HudText>>,
) {
    let value = if let Some((mesh_entity, _hit)) = pointer.get_nearest_hit()
        && let Some(entity) = crate::utils::find(*mesh_entity, &interactable_search)
        && let Ok(interactable) = interactables.get(entity)
    {
        match (interactable, inventory.selection.as_ref()) {
            (Interactable::Item(item), Some(in_hand)) => {
                format!("[RMB] swap {item} with {in_hand}")
            }
            (Interactable::Item(item) | Interactable::Source { item }, None) => {
                format!("[RMB] pick up {item}")
            }
            (Interactable::PlaceableLocation, Some(in_hand)) => format!("[RMB] place {in_hand}"),
            (Interactable::Discardable, Some(in_hand)) => format!("[RMB] discard {in_hand}"),
            (Interactable::Source { .. }, Some(_)) => String::new(),
            (Interactable::PlaceableLocation | Interactable::Discardable, None) => String::new(),
            (Interactable::Drawer, _) => String::from(match drawers.get(entity) {
                Ok(Drawer { opened: true, .. }) => "[RMB] close drawer",
                Ok(Drawer { opened: false, .. }) => "[RMB] open drawer",
                Err(_) => "",
            }),
        }
    } else {
        String::new()
    };
    text.0 = value;
}
