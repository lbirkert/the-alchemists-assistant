use bevy::ui_widgets::Button;

use crate::{prelude::*, ui::FocusUpdate};

const BUTTON_COLOR_PRIMARY: Color = Color::WHITE;
const BUTTON_COLOR_SECONDARY: Color = Color::BLACK;
const BUTTON_COLOR_BACKGROUND: Color = Color::NONE;

#[derive(Component, FromTemplate, Reflect, Clone, Copy)]
#[reflect(Component)]
pub struct MenuButton {
    pub text_entity: Entity,
    pub do_not_offset: bool,
}

pub(super) fn button(label: &'static str) -> impl Scene {
    bsn! {
        #Button
        Name::new(format!("Button {label}"))
        MenuButton {
            text_entity: #ButtonText
        }
        Button
        on(update_menu_buttons)
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(20), px(2))
        }
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
    event: On<FocusUpdate>,
    mut commands: Commands,
    query: Query<(&MenuButton, &BackgroundColor, &Node)>,
    text_query: Query<&TextColor>,
) {
    if let Ok((button, background_color, node)) = query.get(event.entity) {
        let is_active = event.is_focused;

        let color = match is_active {
            true => BUTTON_COLOR_PRIMARY,
            false => BUTTON_COLOR_BACKGROUND,
        };
        let margin_right = match is_active {
            true if button.do_not_offset => px(0),
            true => px(-20),
            false => px(0),
        };
        commands.entity(event.entity).insert((
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
