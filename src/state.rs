use bevy::prelude::*;

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum GameState {
    #[default]
    Title,
    Game,
    End,
}

pub fn state_plugin(app: &mut App) {
    app.init_state::<GameState>();
}
