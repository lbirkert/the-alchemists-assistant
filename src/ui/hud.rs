use crate::{drawer::Drawer, prelude::*, scene::stations::MortarStation};

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
    mortars: Query<&MortarStation>,
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
            (Interactable::ItemPlaceable | Interactable::StickyNotePlaceable, Some(in_hand)) => {
                format!("[RMB] place {in_hand}")
            }
            (Interactable::ItemDiscardable, Some(in_hand)) => format!("[RMB] discard {in_hand}"),
            (Interactable::Source { .. }, Some(in_hand)) => format!("Already holding {in_hand}"),
            (
                Interactable::ItemPlaceable
                | Interactable::StickyNotePlaceable
                | Interactable::ItemDiscardable,
                None,
            ) => String::new(),
            (Interactable::Drawer, _) => String::from(match drawers.get(entity) {
                Ok(Drawer { opened: true, .. }) => "[RMB] close drawer",
                Ok(Drawer { opened: false, .. }) => "[RMB] open drawer",
                Err(_) => "",
            }),
            (Interactable::StickyNote(_), None) => format!("[RMB] pick up Sticky Note"),
            (Interactable::StickyNote(_), Some(in_hand)) => format!("Already holding {in_hand}"),
            (Interactable::Mortar, Some(in_hand)) => match mortars.get(entity) {
                Ok(MortarStation { working: false }) => format!("[RMB] grind {in_hand:?}"),
                Ok(MortarStation { working: true }) | Err(_) => String::new(),
            },
            (
                Interactable::ItemPlaceable | Interactable::ItemDiscardable | Interactable::Mortar,
                None,
            ) => String::new(),
        }
    } else {
        String::new()
    };
    text.0 = value;
}
