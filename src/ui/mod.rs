use std::time::Duration;

use bevy::{
    picking::hover::Hovered,
    ui::Pressed,
    ui_widgets::{Activate, Button},
};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Title), setup);
    app.add_systems(
        // Needs to be post update, see <https://github.com/vleue/bevy_easings/issues/23>
        TitleScreenPostUpdate,
        (ApplyDeferred, update_menu_buttons, ApplyDeferred).chain(),
    );
    app.add_systems(
        GameUpdate,
        set_state_title.run_if(input_just_pressed(KeyCode::KeyM)),
    );
}

const BUTTON_COLOR_PRIMARY: Color = Color::WHITE;
const BUTTON_COLOR_SECONDARY: Color = Color::BLACK;
const BUTTON_COLOR_BACKGROUND: Color = Color::NONE;

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
            DespawnOnExit::<GameState>(GameState::Title)
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
    let settings = commands.spawn_scene(button("Settings")).id();
    let quit = commands
        .spawn_scene(bsn! {
            @button("Quit")
            on(|_event: On<Activate>, mut app_exit: MessageWriter<AppExit>| {
                app_exit.write(AppExit::Success);
            })
        })
        .id();

    let menu_buttons = MenuButtons {
        active: 0,
        buttons: [play, settings, quit],
    };
    commands.entity(title).add_children(&menu_buttons.buttons);

    commands.insert_resource(menu_buttons);
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct MenuButtons {
    active: usize,
    buttons: [Entity; 3],
}

#[derive(Component, FromTemplate, Reflect, Clone, Copy)]
#[reflect(Component)]
struct MenuButton {
    already_active: bool,
    ignore_hover: bool,
    text_entity: Entity,
}

fn button(label: &'static str) -> impl Scene {
    bsn! {
        #Button
        Name::new(format!("Button {label}"))
        MenuButton {
            text_entity: #ButtonText
        }
        Button
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(20), px(2))
        }
        Hovered
        BackgroundColor
        Children [
            #ButtonText
            Text::new(label)
            TextFont {
                font_size: FontSize::Rem(2.)
            }
        ]
    }
}

fn update_menu_buttons(
    mut commands: Commands,
    mut menu: ResMut<MenuButtons>,
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(
        Entity,
        &mut MenuButton,
        &Hovered,
        Has<Pressed>,
        &BackgroundColor,
        &Node,
    )>,
    text_query: Query<&TextColor>,
) {
    if input.any_just_pressed([
        KeyCode::KeyW,
        KeyCode::KeyA,
        KeyCode::ArrowUp,
        KeyCode::KeyK,
    ]) || (input.pressed(KeyCode::ShiftLeft) && input.just_pressed(KeyCode::Tab))
    {
        // Previous
        if menu.active == 0 {
            menu.active = menu.buttons.len() - 1;
        } else {
            menu.active -= 1;
        }
    } else if input.any_just_pressed([
        KeyCode::KeyS,
        KeyCode::KeyD,
        KeyCode::ArrowDown,
        KeyCode::KeyJ,
    ]) || input.just_pressed(KeyCode::Tab)
    {
        // Next
        menu.active = (menu.active + 1) % menu.buttons.len();
    }

    for (entity, mut button, hovered, is_pressed, background_color, node) in query {
        let is_active = menu.buttons[menu.active] == entity
            || (hovered.0 && !button.ignore_hover && !button.already_active)
            || is_pressed;

        button.ignore_hover = hovered.0 && !is_active;

        if is_active != button.already_active {
            button.already_active = is_active;

            if is_active {
                menu.active = menu
                    .buttons
                    .iter()
                    .position(|e| *e == entity)
                    .unwrap_or_default();
            }

            let color = match is_active {
                true => BUTTON_COLOR_PRIMARY,
                false => BUTTON_COLOR_BACKGROUND,
            };
            let margin_right = match is_active {
                true => px(-20),
                false => px(0),
            };
            commands.entity(entity).insert((
                background_color.ease_to(
                    BackgroundColor(color),
                    EaseFunction::CubicIn,
                    EasingType::Once {
                        duration: Duration::from_millis(120),
                    },
                ),
                node.clone().ease_to(
                    {
                        let mut node = node.clone();
                        node.margin.right = margin_right;
                        node
                    },
                    EaseFunction::CubicIn,
                    EasingType::Once {
                        duration: Duration::from_millis(120),
                    },
                ),
            ));

            let color = match is_active {
                true => BUTTON_COLOR_SECONDARY,
                false => BUTTON_COLOR_PRIMARY,
            };
            if let Ok(text_color) = text_query.get(button.text_entity) {
                commands
                    .entity(button.text_entity)
                    .insert(text_color.ease_to(
                        TextColor(color),
                        EaseFunction::CubicIn,
                        EasingType::Once {
                            duration: Duration::from_millis(120),
                        },
                    ));
            }
        }
    }

    // Activate the focused button
    if input.any_just_pressed([KeyCode::KeyE, KeyCode::Enter]) {
        commands
            .entity(menu.buttons[menu.active])
            .trigger(|e| Activate { entity: e });
    }
}

fn set_state_title(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Title);
}
