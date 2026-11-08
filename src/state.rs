use crate::prelude::*;

use bevy::ecs::schedule::ScheduleLabel;

pub fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.init_state::<SettingsOpen>();
    app.add_sub_state::<MainMenuOpen>();
    app.add_sub_state::<GamePaused>();
    app.add_sub_state::<MonthFinishedScreen>();

    app.add_schedule(Schedule::new(TitleScreenUpdate));
    app.add_schedule(Schedule::new(TitleScreenPostUpdate));
    app.add_schedule(Schedule::new(GameUpdate));
    app.add_schedule(Schedule::new(GamePostUpdate));

    app.add_systems(Update, run_update_schedules);
    app.add_systems(PostUpdate, run_post_update_schedules);

    app.add_systems(ON_GAME_EXIT, despawn_game_objects);

    app.add_systems(GameUpdate, set_paused_state);
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum GameState {
    #[default]
    Title,
    Game,
    End,
}

#[derive(SubStates, Default, Debug, Hash, Eq, PartialEq, Clone, Copy)]
#[source(GameState = GameState::Game)]
pub struct GamePaused(pub bool);

#[derive(SubStates, Default, Debug, Hash, Eq, PartialEq, Clone, Copy)]
#[source(GamePaused = GamePaused(true))]
pub struct MonthFinishedScreen(pub bool);

#[derive(SubStates, Debug, Hash, Eq, PartialEq, Clone, Copy)]
#[source(GameState = GameState::Title)]
pub struct MainMenuOpen(pub bool);

impl Default for MainMenuOpen {
    fn default() -> Self {
        MainMenuOpen(true)
    }
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct SettingsOpen(pub bool);

pub const ON_GAME_ENTER: OnEnter<GameState> = OnEnter(GameState::Game);
pub const ON_GAME_EXIT: OnExit<GameState> = OnExit(GameState::Game);

#[derive(ScheduleLabel, Hash, Clone, Copy, Debug, PartialEq, Eq)]
/// Convenience, allows using
/// ```ignore
/// app.add_systems(GameUpdate, systems);
/// ```
/// instead of
/// ```ignore
/// app.add_systems(Update, systems.run_if(in_state(GamePaused(false)));
/// ```
pub struct GameUpdate;

#[derive(ScheduleLabel, Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GamePostUpdate;

#[derive(ScheduleLabel, Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleScreenUpdate;

#[derive(ScheduleLabel, Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleScreenPostUpdate;

fn run_update_schedules(world: &mut World) {
    let Some(game_state) = world.get_resource::<State<GameState>>() else {
        return;
    };
    let is_paused = world
        .get_resource::<State<GamePaused>>()
        .map_or_default(|s| s.get().0);
    let game_state = *game_state.get();

    match game_state {
        GameState::Title => world.run_schedule(TitleScreenUpdate),
        GameState::Game if !is_paused => world.run_schedule(GameUpdate),
        _ => {}
    }
}

fn run_post_update_schedules(world: &mut World) {
    let Some(game_state) = world.get_resource::<State<GameState>>() else {
        return;
    };
    let is_paused = world
        .get_resource::<State<GamePaused>>()
        .map_or_default(|s| s.get().0);
    let game_state = *game_state.get();

    match game_state {
        GameState::Title => world.run_schedule(TitleScreenPostUpdate),
        GameState::Game if !is_paused => world.run_schedule(GamePostUpdate),
        _ => {}
    }
}

fn despawn_game_objects(mut commands: Commands, query: Query<Entity, With<EnvironmentObject>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

fn set_paused_state(
    input: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    state: Res<State<GamePaused>>,
    mut next_state: ResMut<NextState<GamePaused>>,
) {
    if input.just_pressed(bindings.pause_game) {
        let current = state.get().0;
        next_state.set(GamePaused(!current));
    }
}
