use bevy_easings::CustomComponentEase;

use crate::{
    drawer::Drawer,
    intro::{ComputerState, GameIntro},
    manager::DayState,
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

    #[cfg(debug_assertions)]
    app.add_systems(Update, |query: Query<&Cauldron>| {
        if query.count() > 1 {
            panic!(
                "having more than one cauldrons breaks the hud: we have {}",
                query.count()
            );
        }
    });
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
    intro: Res<GameIntro>,
    manager: Res<GameManager>,
    inventory: Option<Single<&Inventory>>,
    drawers: Query<&Drawer>,
    mortars: Query<&ProcessingStation>,
    mut text: Single<&mut Text, With<HudText>>,
    cauldron: Option<Single<&Cauldron>>,
) {
    let value = if let Some((entity, ref interactable)) = focus.interactable {
        match interactable {
            Interactable::Item(item) => match inventory.and_then(|i| i.selection.as_ref()).as_ref()
            {
                Some(Selection::Item(in_hand)) => format!("[RMB] Swap {item} with {in_hand}"),
                Some(Selection::StickyNote(note))
                    if note.recipe.is_some() && item.potion.is_some() =>
                {
                    format!("[RMB] Label {item} with {note}")
                }
                Some(Selection::StickyNote(note))
                    if note.recipe.is_some()
                        && item.potion.as_ref().is_some_and(|p| p.recipe.is_some()) =>
                {
                    format!("{item} already has a recipe")
                }
                Some(Selection::StickyNote(_)) => String::new(),
                None => format!("[RMB] Pick up {item}"),
            },
            Interactable::StickyNote(note) => match inventory.and_then(|i| i.selection.as_ref()) {
                Some(in_hand) => format!("Already holding {in_hand}"),
                None => format!("[RMB] Pick up {note}"),
            },
            Interactable::ItemPlaceable => match inventory.and_then(|i| i.selection.as_ref()) {
                Some(Selection::Item(in_hand)) => format!("[RMB] Place {in_hand}"),
                None | Some(Selection::StickyNote(_)) => String::new(),
            },
            Interactable::StickyNotePlaceable => match inventory.and_then(|i| i.selection.as_ref())
            {
                Some(Selection::StickyNote(note)) => format!("[RMB] Place {note}"),
                None | Some(Selection::Item(_)) => String::new(),
            },
            Interactable::ItemDiscardable => match inventory.and_then(|i| i.selection.as_ref()) {
                Some(Selection::Item(in_hand)) => format!("[RMB] Discard {in_hand}"),
                Some(Selection::StickyNote(_)) => {
                    String::from("This is too important to throw away!")
                }
                None => String::new(),
            },
            Interactable::NpcTable => match inventory.and_then(|i| i.selection.as_ref()) {
                Some(Selection::Item(item))
                    if item.potion.as_ref().is_some_and(|p| p.recipe.is_some()) =>
                {
                    format!("[RMB] Submit {item} - press bell afterwards")
                }
                Some(Selection::Item(Item {
                    kind: ItemKind::Potion,
                    potion: None,
                    ..
                })) => String::from("I will place my potion here after I brewed it."),
                Some(Selection::Item(Item {
                    potion: Some(PotionData { recipe: None, .. }),
                    ..
                })) => String::from("My potion needs a recipe label."),
                _ => String::new(),
            },
            Interactable::Bell => String::from("[RMB] Call alchemist"),
            Interactable::Source { item } => match inventory.and_then(|i| i.selection.as_ref()) {
                Some(in_hand) => format!("Already holding {in_hand}"),
                None => format!("[RMB] Take up {item}"),
            },
            Interactable::Drawer => String::from(match drawers.get(entity) {
                Ok(Drawer { opened: true, .. }) => "[RMB] Close drawer",
                Ok(Drawer { opened: false, .. }) => "[RMB] Open drawer",
                Err(_) => "",
            }),
            Interactable::Station => {
                if let Ok(station) = mortars.get(entity) {
                    match (&station.state, inventory.and_then(|i| i.selection.as_ref())) {
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
            Interactable::Cauldron => match inventory.and_then(|i| i.selection.as_ref()) {
                Some(Selection::Item(item))
                    if item.kind == ItemKind::Potion && item.potion.is_none() =>
                {
                    if cauldron.is_some_and(|c| c.is_empty()) {
                        String::from("Cauldron is empty")
                    } else {
                        String::from("[RMB] Fill Potion")
                    }
                }
                Some(Selection::Item(item)) if item.kind != ItemKind::Potion => {
                    format!("[RMB] Add {item} to Cauldron")
                }
                Some(Selection::Item(item)) => {
                    debug_assert_eq!(item.kind, ItemKind::Potion);
                    debug_assert!(item.potion.is_some());
                    match cauldron.is_some_and(|c| c.is_empty()) {
                        true => String::new(),
                        false => format!("[RMB] {item} is already filled"),
                    }
                }
                None | Some(Selection::StickyNote(_)) => String::new(),
            },
            Interactable::Door => match manager.day_state {
                DayState::Running | DayState::Started => String::new(),
                DayState::Ended => String::from("[RMB] Go to sleep"),
            },
            Interactable::IntroComputer => match intro.state {
                ComputerState::News => String::from("[RMB] Find a new job."),
                ComputerState::FindJob => {
                    String::from("[LMB] Look at the news\n[RMB] Search for a new job")
                }
                ComputerState::Apply => {
                    String::from("[LMB] Go back\n[RMB] Apply as the alchemist's assistant")
                }
                ComputerState::Finish { .. } => String::new(),
            },
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
