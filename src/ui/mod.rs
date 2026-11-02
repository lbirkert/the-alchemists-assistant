use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Title), setup);
    app.add_systems(
        TitleScreenUpdate,
        set_state_game.run_if(input_just_pressed(KeyCode::KeyE)),
    );
    app.add_systems(
        GameUpdate,
        set_state_title.run_if(input_just_pressed(KeyCode::KeyM)),
    );
}

fn setup(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #TitleScreen
        DespawnOnExit::<GameState>(GameState::Title)
    });
}

fn set_state_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Game);
}

fn set_state_title(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Title);
}
