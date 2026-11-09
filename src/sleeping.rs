use bevy_easings::EaseMethod;

use crate::{
    manager::DayState,
    player::{Player, PlayerEyes},
    prelude::*,
    utils::color_overlay::ColorOverlay,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Sleeping>();
    app.add_systems(ON_GAME_ENTER, |mut sleeping: ResMut<Sleeping>| {
        *sleeping = Sleeping::default()
    });

    app.add_systems(
        GameUpdate,
        (sleep, update_sleeping.in_set(PhysicsSystems::Last)),
    );
}

// #[derive(SubStates, Default, Debug, Clone, Hash, PartialEq, Eq)]
// #[source(GameState = GameState::Game)]
#[derive(Resource, Default)]
struct Sleeping {
    timer: Option<(Timer, SleepingState)>,
}

enum SleepingState {
    FallAsleep,
    Sleep,
    WakeUp,
}

const PLAYER_FALL_ASLEEP: Duration = Duration::from_millis(1000);
const PLAYER_SLEEPING: Duration = Duration::from_millis(2000);
const PLAYER_WAKE_UP: Duration = Duration::from_millis(1000);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Interactable::Door)]
#[type_path = "scene"]
struct Door;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[type_path = "positions"]
struct PlayerWakeUpPos;

fn sleep(
    mut messages: MessageReader<PlayerInteraction>,
    mut commands: Commands,
    mut sleeping: ResMut<Sleeping>,
    mut state: ResMut<NextState<CutScene>>,
    mut manager: ResMut<GameManager>,
    overlay: Single<Entity, With<ColorOverlay>>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::Door,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if manager.day_state != DayState::Ended {
        return;
    }

    manager.sleep_through_night();
    sleeping.timer = Some((
        Timer::new(PLAYER_FALL_ASLEEP, TimerMode::Once),
        SleepingState::FallAsleep,
    ));
    state.set(CutScene::Sleeping);
    commands.entity(*overlay).insert(
        BackgroundColor(Color::NONE)
            .ease_to(
                BackgroundColor(Color::BLACK),
                EaseFunction::CubicIn,
                EasingType::Once {
                    duration: PLAYER_FALL_ASLEEP,
                },
            )
            .ease_to(
                BackgroundColor(Color::BLACK),
                EaseMethod::Linear,
                EasingType::Once {
                    duration: PLAYER_SLEEPING,
                },
            )
            .ease_to(
                BackgroundColor(Color::NONE),
                EaseFunction::CubicOut,
                EasingType::Once {
                    duration: PLAYER_WAKE_UP,
                },
            ),
    );
}

fn update_sleeping(
    time: Res<Time>,
    mut sleeping: ResMut<Sleeping>,
    mut cut_scene: ResMut<NextState<CutScene>>,
    mut player: Single<
        &mut Transform,
        (With<Player>, Without<PlayerEyes>, Without<PlayerWakeUpPos>),
    >,
    mut player_eyes: Single<&mut Transform, (With<PlayerEyes>, Without<PlayerWakeUpPos>)>,
    wake_up_pos: Option<Single<&Transform, With<PlayerWakeUpPos>>>,
) {
    if let Some((timer, state)) = sleeping.timer.as_mut() {
        timer.tick(time.delta());
        if timer.is_finished() {
            match state {
                SleepingState::FallAsleep => {
                    sleeping.timer = Some((
                        Timer::new(PLAYER_SLEEPING, TimerMode::Once),
                        SleepingState::Sleep,
                    ));
                    if let Some(pos) = wake_up_pos {
                        player.translation = pos.translation;
                        player_eyes.rotation = pos.rotation;
                    } else {
                        error!("No PlayerWakeUpPos found");
                    }
                }
                SleepingState::Sleep => {
                    sleeping.timer = Some((
                        Timer::new(PLAYER_WAKE_UP, TimerMode::Once),
                        SleepingState::WakeUp,
                    ));
                }
                SleepingState::WakeUp => {
                    sleeping.timer = None;
                    cut_scene.set(CutScene::None);
                }
            }
        }
    }
}
