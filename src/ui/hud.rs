use crate::{
    drawer::Drawer,
    prelude::*,
    scene::stations::{ProcessingStation, StationState},
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
    mortars: Query<&ProcessingStation>,
    mut text: Single<&mut Text, With<HudText>>,
) {
    let value = if let Some((mesh_entity, _hit)) = pointer.get_nearest_hit()
        && let Some(entity) = crate::utils::find_parent(*mesh_entity, &interactable_search)
        && let Ok(interactable) = interactables.get(entity)
    {
        match (interactable, inventory.item) {
            (Interactable::Item(item), Some(in_hand)) => {
                format!("[RMB] Swap {item:?} with {in_hand:?}")
            }
            (Interactable::Item(item) | Interactable::Source { item }, None) => {
                format!("[RMB] Pick up {item:?}")
            }
            (Interactable::PlaceableLocation, Some(in_hand)) => format!("[RMB] Place {in_hand:?}"),
            (Interactable::Discardable, Some(in_hand)) => format!("[RMB] Discard {in_hand:?}"),
            (Interactable::Source { .. }, Some(in_hand)) => format!("Already holding {in_hand:?}"),
            (Interactable::Drawer, _) => String::from(match drawers.get(entity) {
                Ok(Drawer { opened: true, .. }) => "[RMB] Close drawer",
                Ok(Drawer { opened: false, .. }) => "[RMB] Open drawer",
                Err(_) => "",
            }),
            (Interactable::Station, _) => {
                if let Ok(station) = mortars.get(entity) {
                    match (&station.state, inventory.item) {
                        (StationState::Idle, Some(in_hand)) => {
                            format!("[RMB] {} {in_hand:?}", station.base_verb(true))
                        }
                        (
                            StationState::Processing {
                                working: false,
                                item,
                                ..
                            },
                            None,
                        ) => format!("[RMB] Continue {} {item:?}", station.ing_verb()),
                        (
                            StationState::Processing {
                                working: false,
                                item,
                                ..
                            },
                            Some(in_hand),
                        ) => format!(
                            "[RMB] Cannot {} {item:?} while holding {in_hand:?}",
                            station.base_verb(false)
                        ),
                        (StationState::Done { item, .. }, None) => format!("[RMB] Take {item:?}"),
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
