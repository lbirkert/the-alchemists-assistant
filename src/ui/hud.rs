use bevy_easings::CustomComponentEase;

use crate::{
    drawer::Drawer,
    prelude::*,
    scene::stations::{ProcessingStation, StationState},
    upgrades::NewUnlock,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GamePaused(false)), setup);
    app.add_systems(
        GameUpdate,
        (
            update_hud_text,
            update_unlock_messages,
            sync_font_size,
            bevy_easings::custom_ease_system::<Virtual, FontSizeWrapper>,
        ),
    );
}

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
struct HudText;

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
struct UnlockText;

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
            #UnlockText
            Node {
                width: percent(100),
                height: percent(70),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center
            }
            Pickable::IGNORE
            Children [
                UnlockText
                UNLOCK_ZERO_FONT_SIZE
                Text::new("")
                TextFont {
                    weight: FontWeight::BOLD
                }
                Node {
                    padding: UiRect::axes(px(50), px(10))
                }
                BackgroundColor(Color::srgba(0., 0., 0., 0.))
                Pickable::IGNORE
            ]
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
    focus: Res<PlayerFocus>,
    inventory: Single<&Inventory>,
    drawers: Query<&Drawer>,
    mortars: Query<&ProcessingStation>,
    mut text: Single<&mut Text, With<HudText>>,
    cauldron: Single<&Cauldron>,
) {
    let value = if let Some((entity, ref interactable)) = focus.interactable {
        match (interactable, inventory.selection.as_ref()) {
            (Interactable::Item(item), Some(Selection::Item(in_hand))) => {
                format!("[RMB] Swap {item} with {in_hand}")
            }
            (Interactable::Item(item) | Interactable::Source { item }, None) => {
                format!("[RMB] Pick up {item}")
            }
            (Interactable::ItemPlaceable, Some(Selection::Item(in_hand))) => {
                format!("[RMB] Place {in_hand}")
            }
            (Interactable::StickyNotePlaceable, Some(Selection::StickyNote(_))) => {
                String::from("[RMB] Place Sticky Note")
            }
            (Interactable::ItemDiscardable, Some(Selection::Item(in_hand))) => {
                format!("[RMB] Discard {in_hand}")
            }
            (
                Interactable::NpcTable,
                Some(Selection::Item(Item {
                    potion:
                        Some(PotionData {
                            recipe: Some(..), ..
                        }),
                    ..
                })),
            ) => {
                format!("[RMB] Submit Potion - press bell afterwards")
            }
            (
                Interactable::Item(Item {
                    potion: Some(PotionData { recipe: None, .. }),
                    ..
                }),
                Some(Selection::StickyNote(StickyNote {
                    recipe: Some(..), ..
                })),
            ) => {
                format!("[RMB] Label potion with Recipe")
            }
            (Interactable::Source { .. }, Some(in_hand)) => format!("Already holding {in_hand}"),
            (
                Interactable::ItemPlaceable
                | Interactable::StickyNotePlaceable
                | Interactable::ItemDiscardable,
                None,
            ) => String::new(),
            (Interactable::Drawer, _) => String::from(match drawers.get(entity) {
                Ok(Drawer { opened: true, .. }) => "[RMB] Close drawer",
                Ok(Drawer { opened: false, .. }) => "[RMB] Open drawer",
                Err(_) => "",
            }),
            (Interactable::StickyNote(_), None) => String::from("[RMB] Pick up Sticky Note"),
            (Interactable::StickyNote(_), Some(in_hand)) => format!("Already holding {in_hand}"),
            (Interactable::Bell, _) => format!("[RMB] call alchemist"),
            (
                Interactable::Cauldron,
                Some(Selection::Item(Item {
                    kind: ItemKind::Potion,
                    potion: None,
                    ..
                })),
            ) => {
                if !cauldron.is_empty() {
                    format!("[RMB] Fill Potion")
                } else {
                    String::new()
                }
            }
            (Interactable::Cauldron, Some(Selection::Item(..))) => format!("[RMB] Add to Cauldron"),
            (Interactable::Station, _) => {
                if let Ok(station) = mortars.get(entity) {
                    match (&station.state, inventory.selection.as_ref()) {
                        (StationState::Idle, Some(Selection::Item(in_hand))) => {
                            format!("[RMB] {} {in_hand}", station.base_verb(true))
                        }
                        (
                            StationState::Processing {
                                working: false,
                                item,
                                ..
                            },
                            None,
                        ) => format!("[RMB] Continue {} {item}", station.ing_verb()),
                        (
                            StationState::Processing {
                                working: false,
                                item,
                                ..
                            },
                            Some(in_hand),
                        ) => format!(
                            "[RMB] Cannot {} {item} while holding {in_hand}",
                            station.base_verb(false)
                        ),
                        (StationState::Done { item, .. }, None) => {
                            format!("[RMB] Take {}", station.kind.process_item(item.clone()))
                        }
                        _ => String::new(),
                    }
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    } else {
        String::new()
    };
    text.0 = value;
}

#[derive(Component, Default, Clone)]
struct FontSizeWrapper(FontSize);

impl bevy_easings::Lerp for FontSizeWrapper {
    type Scalar = f32;
    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        let inner = match (self.0, other.0) {
            (FontSize::Rem(x), FontSize::Rem(y)) => FontSize::Rem(x.lerp(y, *scalar)),
            _ => unimplemented!(),
        };
        Self(inner)
    }
}

const UNLOCK_ZERO_FONT_SIZE: FontSizeWrapper = FontSizeWrapper(FontSize::Rem(0.));
const UNLOCK_FONT_SIZE: FontSizeWrapper = FontSizeWrapper(FontSize::Rem(1.6));

const UNLOCK_ZERO_BACKGROUND: BackgroundColor = BackgroundColor(Color::srgba(0., 0., 0., 0.));
const UNLOCK_BACKGROUND: BackgroundColor = BackgroundColor(Color::srgba(0., 0., 0., 0.8));

fn update_unlock_messages(
    time: Res<Time>,
    mut commands: Commands,
    mut messages: MessageReader<NewUnlock>,
    mut active_timer: Local<Option<(Timer, bool)>>,
    mut queue: Local<VecDeque<NewUnlock>>,
    mut text: Single<(Entity, &mut Text), With<UnlockText>>,
) {
    let (entity, text) = &mut *text;

    for unlock in messages.read() {
        queue.push_back(*unlock);
    }

    if let Some((timer, fade_out)) = &mut *active_timer {
        timer.tick(time.delta());
        if timer.remaining_secs() < 0.5 && *fade_out {
            *fade_out = false;
            commands.entity(*entity).insert((
                UNLOCK_FONT_SIZE.ease_to(
                    UNLOCK_ZERO_FONT_SIZE,
                    EaseFunction::CubicOut,
                    EasingType::Once {
                        duration: Duration::from_millis(200),
                    },
                ),
                UNLOCK_BACKGROUND.ease_to(
                    UNLOCK_ZERO_BACKGROUND,
                    EaseFunction::CubicOut,
                    EasingType::Once {
                        duration: Duration::from_millis(200),
                    },
                ),
            ));
        } else if timer.is_finished() {
            *active_timer = None;
            text.0 = String::new();
        }
    } else if let Some(unlock) = queue.pop_front() {
        *active_timer = Some((
            Timer::new(Duration::from_millis(5000), TimerMode::Once),
            true,
        ));
        commands.entity(*entity).insert((
            UNLOCK_ZERO_FONT_SIZE.ease_to(
                UNLOCK_FONT_SIZE,
                EaseFunction::SineIn,
                EasingType::Once {
                    duration: Duration::from_millis(500),
                },
            ),
            UNLOCK_ZERO_BACKGROUND.ease_to(
                UNLOCK_BACKGROUND,
                EaseFunction::SineIn,
                EasingType::Once {
                    duration: Duration::from_millis(500),
                },
            ),
        ));
        text.0 = match unlock {
            NewUnlock::Item(item) => format!("New Item Unlocked: {item:?}"),
            NewUnlock::Station(station) => format!("New Station Unlocked: {station:?}"),
        }
    }
}

fn sync_font_size(query: Query<(&mut TextFont, &FontSizeWrapper), Changed<FontSizeWrapper>>) {
    for (mut text, wrapper) in query {
        text.font_size = wrapper.0;
    }
}
