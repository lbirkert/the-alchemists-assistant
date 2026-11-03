use crate::{
    prelude::*,
    ui::{FocusableItems, button},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(SettingsOpen(true)), setup);
    app.add_systems(OnExit(SettingsOpen(true)), |mut commands: Commands| {
        commands.remove_resource::<FocusableItems>();
    });
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
                align_items: AlignItems::End,
                justify_content: JustifyContent::Center,
                padding: UiRect {
                    right: percent(12)
                }
            }
            DespawnOnExit::<SettingsOpen>(SettingsOpen(true))
        })
        .id();

    let back = commands
        .spawn_scene(bsn! {
            @button("Back")
            on(close_settings)
        })
        .id();

    let focusable = FocusableItems::new([back]);
    commands.entity(menu).add_children(&focusable.items);

    commands.insert_resource(focusable);
}

fn close_settings(
    _event: On<Activate>,
    mut menu: Option<ResMut<NextState<MainMenuOpen>>>,
    mut settings: ResMut<NextState<SettingsOpen>>,
) {
    if let Some(menu) = &mut menu {
        menu.set(MainMenuOpen(true));
    }
    settings.set(SettingsOpen(false));
}
