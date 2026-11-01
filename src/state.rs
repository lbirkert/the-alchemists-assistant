use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

pub fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.add_sub_state::<GamePaused>();

    app.add_schedule(Schedule::new(GameRunning));

    app.add_systems(
        Update,
        (|world: &mut World| world.run_schedule(GameRunning)).run_if(in_state(GamePaused(false))),
    );
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum GameState {
    // TODO! Title should be default
    Title,
    #[default]
    Game,
    End,
}

#[derive(SubStates, Default, Debug, Hash, Eq, PartialEq, Clone)]
#[source(GameState = GameState::Game)]
pub struct GamePaused(pub bool);

pub const ON_GAME_ENTER: OnEnter<GameState> = OnEnter(GameState::Game);
pub const ON_GAME_EXIT: OnExit<GameState> = OnExit(GameState::Game);

#[derive(ScheduleLabel, Hash, Clone, Copy, Debug, PartialEq, Eq)]
/// Convenience, allows using
/// ```ignore
/// app.add_systems(GameRunning, systems);
/// ```
/// instead of
/// ```ignore
/// app.add_systems(Update, systems.run_if(in_state(GamePaused(false)));
/// ```
pub struct GameRunning;
