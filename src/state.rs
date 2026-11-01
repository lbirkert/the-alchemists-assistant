use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.add_sub_state::<GamePaused>();
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum GameState {
    #[default]
    Title,
    Game,
    End,
}

#[derive(SubStates, Default, Debug, Hash, Eq, PartialEq, Clone)]
#[source(GameState = GameState::Game)]
pub struct GamePaused(pub bool);

pub fn normal_game_state(/*TODO*/) -> bool {
    false
}
