use crate::{
    prelude::*,
    ui::{
        FocusableItems,
        button::{MenuButton, button},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(TitleScreenState::Credits), setup);
    app.add_systems(
        OnExit(TitleScreenState::Credits),
        |mut commands: Commands| {
            commands.remove_resource::<FocusableItems>();
        },
    );
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
            DespawnOnExit::<TitleScreenState>(TitleScreenState::Credits)
            Children [
                @section_node("Code", &["lbirkert", "faervan"], 1.)
                --
                @section_node("Third Party 3D Models", &get_assets(), 0.8)
                --
                @section_node("Other 3D Models", &["lbirkert", "faervan"], 1.)
                --
                @section_node("Dialogues/Narration", &["lbirkert"], 1.)
                --
                @section_node("Sound Effects", &["faervan"], 1.)
            ]
        })
        .id();

    let back = commands
        .spawn_scene(bsn! {
            @button("Back")
            MenuButton {
                do_not_offset: true
            }
            on(close_credits)
        })
        .id();

    let focusable = FocusableItems::new([back]);
    commands.entity(menu).add_children(&focusable.items);

    commands.insert_resource(focusable);
}

fn section_node(label: &'static str, items: &[impl ToString], font_size_rem: f32) -> impl Scene {
    let items: Vec<_> = items
        .iter()
        .map(|v| credit(v.to_string(), font_size_rem))
        .collect();
    let items = bsn_list! {
        {items}
    };
    bsn! {
        Name::new(label)
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            align_items: AlignItems::Center,
        }
        Children [
            @section(label)
            --
            { items }
        ]
    }
}

fn section(label: impl ToString) -> impl Scene {
    let label = label.to_string();
    bsn! {
        Name::new(label.clone())
        Text::new(label)
        TextFont {
            font_size: rem(1.6)
        }
    }
}

fn credit(text: impl ToString, font_size_rem: f32) -> impl Scene {
    let text = text.to_string();
    bsn! {
        #Credit
        Text::new(text)
        TextFont {
            font_size: rem(font_size_rem)
        }
    }
}

fn close_credits(_event: On<Activate>, mut title: Option<ResMut<NextState<TitleScreenState>>>) {
    if let Some(title) = &mut title {
        title.set(TitleScreenState::MainMenu);
    }
}

fn get_assets() -> Vec<String> {
    let asset_md = include_str!("../../ASSETS.md");
    asset_md
        .lines()
        .skip_while(|line| *line != "--|--|--")
        .skip(1)
        .take_while(|line| !line.is_empty())
        .map(|line| line.split('|'))
        .filter_map(|split| {
            if let [url, license, desc] = split.collect::<Vec<_>>()[..] {
                let desc = desc.trim_end_matches('.');
                Some(format!("{desc} ({license}): {url}"))
            } else {
                None
            }
        })
        .collect()
}
