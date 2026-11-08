use bevy::audio::Volume;

use crate::{
    prelude::*,
    ui::{
        FocusableItems,
        button::{MenuButton, button},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(SettingsOpen(true)), setup);
    app.add_systems(OnExit(SettingsOpen(true)), |mut commands: Commands| {
        commands.remove_resource::<FocusableItems>();
    });
    app.add_systems(Update, update_volume.run_if(in_state(SettingsOpen(true))));
}

fn setup(mut commands: Commands) {
    let menu = commands
        .spawn_scene(bsn! {
            #TitleScreen
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
            BackgroundGradient::from(LinearGradient {
                stops: vec![
                    ColorStop::new(Color::NONE, percent(0)),
                    ColorStop::new(Color::NONE, percent(15)),
                    ColorStop::new(Color::srgba(0., 0., 0., 0.825), percent(30)),
                    ColorStop::new(Color::srgba(0., 0., 0., 0.825), percent(70)),
                    ColorStop::new(Color::NONE, percent(85)),
                    ColorStop::new(Color::NONE, percent(100)),
                ],
                angle: PI / 2.,
                ..Default::default()
            })
            DespawnOnExit::<SettingsOpen>(SettingsOpen(true))
        })
        .id();

    let label = commands
        .spawn_scene(bsn! {
            #GlobalVolume
            Text::new("Global Volume")
            TextFont {
                font_size: rem(1.8)
            }
        })
        .id();

    let volume = commands
        .spawn_scene(bsn! {
            #VolumeNode
            Node {
                width: px(200),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(20),
            }
        })
        .id();

    let plus = commands
        .spawn_scene(bsn! {
            #Plus
            @button("+")
            Node {
                padding: UiRect::axes(px(4), px(-8))
            }
            MenuButton {
                do_not_offset: true
            }
            on(increment_volume)
        })
        .id();

    let current = commands
        .spawn_scene(bsn! {
            #CurrentVolume
            Text
            CurrentVolume
        })
        .id();

    let minus = commands
        .spawn_scene(bsn! {
            #Minus
            @button("-")
            Node {
                padding: UiRect::axes(px(4), px(-8))
            }
            MenuButton {
                do_not_offset: true
            }
            on(decrement_volume)
        })
        .id();

    commands
        .entity(volume)
        .add_children(&[plus, current, minus]);

    let back = commands
        .spawn_scene(bsn! {
            @button("Back")
            MenuButton {
                do_not_offset: true
            }
            on(close_settings)
        })
        .id();

    let focusable = FocusableItems::new([back, plus, minus]);
    commands
        .entity(menu)
        .add_children(&[label, volume])
        .add_children(&[back]);

    commands.insert_resource(focusable);
}

#[derive(Component, Default, Clone, Copy)]
struct CurrentVolume;

fn update_volume(volume: Res<GlobalVolume>, query: Query<&mut Text, With<CurrentVolume>>) {
    for mut text in query {
        if let Volume::Linear(v) = volume.volume {
            text.0 = format!("{}%", (v * 100.).round() as usize);
        }
    }
}

fn increment_volume(_event: On<Activate>, mut volume: ResMut<GlobalVolume>) {
    if let Volume::Linear(v) = &mut volume.volume {
        *v += 0.05;
    } else {
        error!("Expected linear volume")
    }
}

fn decrement_volume(_event: On<Activate>, mut volume: ResMut<GlobalVolume>) {
    if let Volume::Linear(v) = &mut volume.volume
        && *v >= 0.05
    {
        *v -= 0.05;
    } else {
        error!("Expected linear volume")
    }
}

fn close_settings(
    _event: On<Activate>,
    mut title: Option<ResMut<NextState<TitleScreenState>>>,
    mut settings: ResMut<NextState<SettingsOpen>>,
) {
    if let Some(title) = &mut title {
        title.set(TitleScreenState::MainMenu);
    }
    settings.set(SettingsOpen(false));
}
