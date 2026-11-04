use crate::{
    drawer::Drawer,
    prelude::*,
    scene::stations::{MortarState, MortarStation},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GamePaused(false)), setup);
    app.add_systems(GameUpdate, update_hud_text);
}

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
struct HudText;

const CROSSHAIR_SIZE: f32 = 20.;
const CROSSHAIR_THICKNESS: f32 = 2.;

fn setup(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #GameHud
        Node {
            width: percent(100),
            height: percent(100),
        }
        Pickable::IGNORE
        DespawnOnExit::<GamePaused>(GamePaused(false))
        Children [
            HudText
            Node {
                position_type: PositionType::Absolute,
                right: px(20),
                bottom: px(10),
            }
            Text::new("")
            --
            #CrossHair
            Node {
                width: px(CROSSHAIR_SIZE),
                height: px(CROSSHAIR_SIZE),
                top: percent(50),
                left: percent(50),
                margin: UiRect {
                    top: px(-CROSSHAIR_SIZE / 2.),
                    left: px(-CROSSHAIR_SIZE / 2.),
                },
                position_type: PositionType::Absolute,
            }
            Pickable::IGNORE
            Children [
                Node {
                    width: percent(100),
                    height: px(CROSSHAIR_THICKNESS),
                    top: px(CROSSHAIR_SIZE / 2. - CROSSHAIR_THICKNESS / 2.),
                    position_type: PositionType::Absolute,
                }
                BackgroundColor(Color::WHITE)
                Pickable::IGNORE
                --
                Node {
                    width: px(CROSSHAIR_THICKNESS),
                    height: percent(100),
                    left: px(CROSSHAIR_SIZE / 2. - CROSSHAIR_THICKNESS / 2.),
                    position_type: PositionType::Absolute,
                }
                BackgroundColor(Color::WHITE)
                Pickable::IGNORE
            ]
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
        && let Some(entity) = crate::utils::find_parent(*mesh_entity, &interactable_search)
        && let Ok(interactable) = interactables.get(entity)
    {
        match (interactable, inventory.item) {
            (Interactable::Item(item), Some(in_hand)) => {
                format!("[RMB] swap {item:?} with {in_hand:?}")
            }
            (Interactable::Item(item) | Interactable::Source { item }, None) => {
                format!("[RMB] pick up {item:?}")
            }
            (Interactable::PlaceableLocation, Some(in_hand)) => format!("[RMB] place {in_hand:?}"),
            (Interactable::Discardable, Some(in_hand)) => format!("[RMB] discard {in_hand:?}"),
            (Interactable::Source { .. }, Some(in_hand)) => format!("Already holding {in_hand:?}"),
            (Interactable::Drawer, _) => String::from(match drawers.get(entity) {
                Ok(Drawer { opened: true, .. }) => "[RMB] close drawer",
                Ok(Drawer { opened: false, .. }) => "[RMB] open drawer",
                Err(_) => "",
            }),
            (Interactable::Mortar, _) => {
                if let Ok(mortar) = mortars.get(entity) {
                    match (&mortar.state, inventory.item) {
                        (MortarState::Idle, Some(in_hand)) => format!("[RMB] pestle {in_hand:?}"),
                        (
                            MortarState::Processing {
                                working: false,
                                item,
                                ..
                            },
                            None,
                        ) => format!("[RMB] continue pestling {item:?}"),
                        (
                            MortarState::Processing {
                                working: false,
                                item,
                                ..
                            },
                            Some(in_hand),
                        ) => format!("[RMB] cannot pestle {item:?} while holding {in_hand:?}"),
                        (MortarState::Done { item, .. }, None) => format!("[RMB] take {item:?}"),
                        _ => String::new(),
                    }
                } else {
                    String::new()
                }
            }
            (Interactable::PlaceableLocation | Interactable::Discardable, None) => String::new(),
        }
    } else {
        String::new()
    };
    text.0 = value;
}
