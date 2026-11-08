use bevy::ui_widgets::Activate;

use crate::prelude::*;

mod button;
use button::button;

mod focus;
use focus::{FocusUpdate, FocusableItems};

mod month_finished;
mod pause;
mod settings;

mod hud;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        focus::plugin,
        settings::plugin,
        pause::plugin,
        month_finished::plugin,
        hud::plugin,
    ));

    app.add_systems(OnEnter(MainMenuOpen(true)), setup);
    app.add_systems(OnExit(MainMenuOpen(true)), |mut commands: Commands| {
        commands.remove_resource::<FocusableItems>();
    });
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
            DespawnOnExit::<MainMenuOpen>(MainMenuOpen(true))
        })
        .id();

    let play = commands
        .spawn_scene(bsn! {
            @button("Play")
            on(|_event: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                next_state.set(GameState::Game);
            })
        })
        .id();
    let settings = commands.spawn_scene(bsn! {
        @button("Settings")
        on(|_event: On<Activate>, mut menu: ResMut<NextState<MainMenuOpen>>, mut settings: ResMut<NextState<SettingsOpen>>| {
            menu.set(MainMenuOpen(false));
            settings.set(SettingsOpen(true));
        })
    }).id();
    let quit = commands
        .spawn_scene(bsn! {
            @button("Quit")
            on(|_event: On<Activate>, mut app_exit: MessageWriter<AppExit>| {
                app_exit.write(AppExit::Success);
            })
        })
        .id();

    let focusable = FocusableItems::new([play, settings, quit]);
    commands.entity(title).add_children(&focusable.items);

    commands.insert_resource(focusable);
}
