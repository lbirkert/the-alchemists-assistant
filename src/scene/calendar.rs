use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, update_calendar_text);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add)]
#[type_path = "the_alchemist_assistant::scene::calendar"]
struct CalendarBlock;

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
struct CalendarBlockWeekdayText;

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
struct CalendarBlockDayText;

impl CalendarBlock {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let current_state = *world.resource::<State<GameState>>().get();

        // This is the texture that will be rendered to.
        let image = Image::new_target_texture(
            256,
            256,
            bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
            None,
        );

        let mut images = world.resource_mut::<Assets<Image>>();
        let image_handle = images.add(image);

        let texture_camera = world
            .commands()
            .spawn((
                Name::new("Calendar Text Camera"),
                Camera2d,
                Camera {
                    // render before the "main pass" camera
                    order: -1,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
                bevy::camera::RenderTarget::Image(image_handle.clone().into()),
                DespawnOnExit(current_state),
            ))
            .id();

        world.commands().spawn_scene(bsn! {
            #CalendarBlockTextNode
            Node {
                // Cover the whole image
                width: percent(100),
                height: percent(100),
                margin: UiRect {
                    top: px(21),
                    left: px(4)
                },
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Start,
                row_gap: px(72),
                align_items: AlignItems::Center,
            }
            UiTargetCamera(texture_camera)
            DespawnOnExit::<GameState>(current_state)
            Children [
                #CalendarBlockWeekdayText
                Text::new("")
                TextFont {
                    font_size: px(40),
                    weight: FontWeight::EXTRA_BOLD
                }
                TextColor(Color::srgba(0.5, 0.5, 0.5, 1.))
                CalendarBlockWeekdayText
                --
                #CalendarBlockDayText
                Text::new("")
                TextFont {
                    font_size: px(40),
                    weight: FontWeight::EXTRA_BOLD
                }
                TextColor(Color::srgba(0.5, 0.5, 0.5, 1.))
                CalendarBlockDayText
            ]
        });

        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh_handle = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.)));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(image_handle),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.,
            unlit: false,
            ..default()
        });

        world.commands().entity(ctx.entity).apply_scene(bsn! {
            Mesh3d(mesh_handle)
            MeshMaterial3d::<StandardMaterial>(material_handle)
            NotShadowCaster
            Transform::from_xyz(0., 0.112, 0.)
        });
    }
}

fn update_calendar_text(
    manager: Res<GameManager>,
    mut next_day: Local<usize>,
    week_days: Query<&mut Text, With<CalendarBlockWeekdayText>>,
    days: Query<
        &mut Text,
        (
            With<CalendarBlockDayText>,
            Without<CalendarBlockWeekdayText>,
        ),
    >,
) {
    // Only update when a new day has started.
    if manager.day < *next_day {
        return;
    }
    *next_day += 1;

    for mut text in week_days {
        text.0 = manager.game_week_day().to_string();
    }

    for mut text in days {
        text.0 = format!(
            "{:02} {}",
            manager.game_day_of_month(),
            &manager.game_month_str()[..3]
        );
    }
}
