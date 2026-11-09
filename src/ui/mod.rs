use bevy::ui_widgets::Activate;

use crate::{intro::GameIntro, prelude::*};

mod button;
use button::button;

mod focus;
use focus::{FocusUpdate, FocusableItems};

mod credits;
mod menu_background;
mod month_finished;
mod pause;
mod settings;

mod hud;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        focus::plugin,
        settings::plugin,
        credits::plugin,
        pause::plugin,
        month_finished::plugin,
        hud::plugin,
        menu_background::plugin,
    ));

    app.add_systems(OnEnter(TitleScreenState::MainMenu), setup);
    app.add_systems(
        OnExit(TitleScreenState::MainMenu),
        |mut commands: Commands| {
            commands.remove_resource::<FocusableItems>();
        },
    );
}

fn setup(mut commands: Commands) {
    let title = commands
        .spawn_scene(bsn! {
            #TitleScreen
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
                    ColorStop::new(Color::NONE, percent(64)),
                    ColorStop::new(Color::srgba(0., 0., 0., 0.825), percent(69)),
                    ColorStop::new(Color::srgba(0., 0., 0., 0.825), percent(91)),
                    ColorStop::new(Color::NONE, percent(96)),
                    ColorStop::new(Color::NONE, percent(100)),
                ],
                angle: PI / 2.,
                ..Default::default()
            })
            DespawnOnExit::<TitleScreenState>(TitleScreenState::MainMenu)
        })
        .id();

    let play = commands
        .spawn_scene(bsn! {
            @button("Play")
            on(set_state_game)
        })
        .id();
    let credits = commands
        .spawn_scene(bsn! {
            @button("Credits")
            on(|_event: On<Activate>, mut next_state: ResMut<NextState<TitleScreenState>>| {
                next_state.set(TitleScreenState::Credits);
            })
        })
        .id();
    let settings = commands.spawn_scene(bsn! {
        @button("Settings")
        on(|_event: On<Activate>, mut title: ResMut<NextState<TitleScreenState>>, mut settings: ResMut<NextState<SettingsOpen>>| {
            title.set(TitleScreenState::Settings);
            settings.set(SettingsOpen(true));
        })
    }).id();

    commands.spawn_scene(bsn! {
        Text::new("The Alchemist's Assistant".to_string())
        DespawnOnExit::<TitleScreenState>(TitleScreenState::MainMenu)
        ChildOf(title)
    });

    #[cfg(not(target_family = "wasm"))]
    let quit = commands
        .spawn_scene(bsn! {
            @button("Quit")
            on(|_event: On<Activate>, mut app_exit: MessageWriter<AppExit>| {
                app_exit.write(AppExit::Success);
            })
        })
        .id();

    #[cfg(not(target_family = "wasm"))]
    let focusable = FocusableItems::new([play, credits, settings, quit]);
    #[cfg(target_family = "wasm")]
    let focusable = FocusableItems::new([play, credits, settings]);
    commands.entity(title).add_children(&focusable.items);

    commands.insert_resource(focusable);
}

fn set_state_game(
    _event: On<Activate>,
    intro: Res<GameIntro>,
    mut next_state: ResMut<NextState<GameState>>,
    mut scene: ResMut<NextState<GameScene>>,
) {
    next_state.set(GameState::Game);
    scene.set(match intro.finished {
        true => GameScene::Game,
        false => GameScene::Intro,
    });
}
