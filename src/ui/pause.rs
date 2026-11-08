use crate::{
    prelude::*,
    state::MonthFinishedScreen,
    ui::{FocusableItems, button},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GamePaused(true)),
        setup.run_if(not(in_state(MonthFinishedScreen(true)))),
    );
    app.add_systems(
        OnEnter(SettingsOpen(false)),
        setup.run_if(in_state(GamePaused(true))),
    );
    app.add_systems(OnExit(GamePaused(true)), |mut commands: Commands| {
        commands.remove_resource::<FocusableItems>();
    });
    app.add_systems(
        OnExit(SettingsOpen(false)),
        (|mut commands: Commands| {
            commands.remove_resource::<FocusableItems>();
        })
        .run_if(in_state(GamePaused(true))),
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
                align_items: AlignItems::End,
                justify_content: JustifyContent::Center,
                padding: UiRect {
                    right: percent(12)
                }
            }
            BackgroundGradient::from(LinearGradient {
                stops: vec![
                    ColorStop::new(Color::NONE, percent(0)),
                    ColorStop::new(Color::NONE, percent(62)),
                    ColorStop::new(Color::srgba(0., 0., 0., 0.825), percent(67)),
                    ColorStop::new(Color::srgba(0., 0., 0., 0.825), percent(91)),
                    ColorStop::new(Color::NONE, percent(96)),
                    ColorStop::new(Color::NONE, percent(100)),
                ],
                angle: PI / 2.,
                ..Default::default()
            })
            DespawnOnEnter::<SettingsOpen>(SettingsOpen(true))
            DespawnOnExit::<GamePaused>(GamePaused(true))
        })
        .id();

    let r#continue = commands
        .spawn_scene(bsn! {
            @button("Continue")
            on(close_pause_menu)
        })
        .id();
    let open_settings = commands
        .spawn_scene(bsn! {
            @button("Settings")
            on(|_event: On<Activate>, mut next_state: ResMut<NextState<SettingsOpen>>| {
                next_state.set(SettingsOpen(true));
            })
        })
        .id();
    let return_to_menu = commands
        .spawn_scene(bsn! {
            @button("Return to Menu")
            on(return_to_menu)
        })
        .id();

    let focusable = FocusableItems::new([r#continue, open_settings, return_to_menu]);
    commands.entity(menu).add_children(&focusable.items);

    commands.insert_resource(focusable);
}

fn close_pause_menu(_event: On<Activate>, mut paused: ResMut<NextState<GamePaused>>) {
    paused.set(GamePaused(false));
}

fn return_to_menu(
    _event: On<Activate>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<NextState<SettingsOpen>>,
) {
    next_state.set(GameState::Title);
    settings.set(SettingsOpen(false));
}
