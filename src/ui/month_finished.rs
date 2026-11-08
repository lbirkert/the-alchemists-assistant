use crate::{
    prelude::*,
    state::MonthFinishedScreen,
    ui::{FocusableItems, button},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MonthFinishedScreen(true)), setup);
    app.add_systems(
        OnExit(MonthFinishedScreen(true)),
        |mut commands: Commands| {
            commands.remove_resource::<FocusableItems>();
        },
    );
}

fn setup(mut commands: Commands) {
    let menu = commands
        .spawn_scene(bsn! {
            #PausedScreen
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect {
                    right: percent(12)
                }
            }
            DespawnOnExit::<MonthFinishedScreen>(MonthFinishedScreen(true))
        })
        .id();

    let r#continue = commands
        .spawn_scene(bsn! {
            @button("Continue playing")
            on(close_pause_menu)
        })
        .id();
    let return_to_menu = commands
        .spawn_scene(bsn! {
            @button("Return to Menu")
            on(return_to_menu)
        })
        .id();

    let focusable = FocusableItems::new([r#continue, return_to_menu]);
    commands.entity(menu).add_children(&focusable.items);

    commands.insert_resource(focusable);
}

fn close_pause_menu(_event: On<Activate>, mut paused: ResMut<NextState<GamePaused>>) {
    paused.set(GamePaused(false));
}

fn return_to_menu(_event: On<Activate>, mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Title);
}
