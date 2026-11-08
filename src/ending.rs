use crate::{prelude::*, state::MonthFinishedScreen};

pub(super) fn plugin(app: &mut App) {
    app.add_message::<MonthEnded>();

    app.add_systems(Startup, setup);

    app.add_systems(ON_GAME_ENTER, |mut ending: ResMut<GameEnding>| {
        ending.month = None;
    });

    app.add_systems(
        Update,
        (
            update_ending_badge.run_if(resource_changed::<GameEnding>),
            read_month_ended,
        ),
    );
}

#[derive(Message)]
pub struct MonthEnded(pub &'static str);

#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
struct GameEnding {
    image: Handle<Image>,
    month: Option<&'static str>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add)]
struct GameEndingBadge;

impl GameEndingBadge {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let image_handle = world.resource::<GameEnding>().image.clone();

        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh_handle = meshes.add(Plane3d::new(Vec3::NEG_X, Vec2::new(0.2, 0.2)));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(image_handle),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.,
            unlit: false,
            ..default()
        });

        world.commands().spawn_scene(bsn! {
            #GameEndingBadge
            ChildOf({ctx.entity})
            Mesh3d(mesh_handle)
            MeshMaterial3d::<StandardMaterial>(material_handle)
            NotShadowCaster
            Transform::from_xyz(0., -0.01, 0.)
        });
    }
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
struct GameEndingMonthText;

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
struct GameEndingBadgeNode;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let width = 256;
    let height = 256;

    // This is the texture that will be rendered to.
    let image = Image::new_target_texture(
        width,
        height,
        bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
        None,
    );

    let image_handle = images.add(image);

    let texture_camera = commands
        .spawn((
            Name::new("Game Ending Badge Text Camera"),
            Camera2d,
            Camera {
                // render before the "main pass" camera
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            bevy::camera::RenderTarget::Image(image_handle.clone().into()),
        ))
        .id();

    commands.spawn_scene(bsn! {
        #GameEndingBadgeNode
        GameEndingBadgeNode
        Node {
            width: percent(100),
            height: percent(100),
            border_radius: percent(50),
            align_items: AlignItems::Center
        }
        BackgroundColor(Color::srgb_u8(232, 97, 0))
        UiTransform::from_rotation(Rot2::radians(PI / 4.))
        UiTargetCamera(texture_camera)
        Visibility::Hidden
        Children [
            #GameEndingMonthText
            GameEndingMonthText
            Text::new("")
            TextFont {
                font_size: px(40)
            }
            TextLayout {
                justify: Justify::Center
            }
            TextColor(Color::WHITE)
            Node {
                width: percent(100),
            }
            --
            #GameEndingText
            Text::new("Employee\nof the Month")
            TextFont {
                font_size: px(25)
            }
            TextLayout {
                justify: Justify::Center
            }
            TextColor(Color::WHITE)
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect {
                    top: px(45),
                },
                position_type: PositionType::Absolute,
            }
        ]
    });

    commands.insert_resource(GameEnding {
        image: image_handle,
        month: None,
    });
}

fn read_month_ended(
    mut ending: ResMut<GameEnding>,
    mut reader: MessageReader<MonthEnded>,
    mut paused: ResMut<NextState<GamePaused>>,
    mut month_finished: ResMut<NextState<MonthFinishedScreen>>,
) {
    if let Some(MonthEnded(month)) = reader.read().next() {
        ending.month = Some(month);
        paused.set(GamePaused(true));
        month_finished.set(MonthFinishedScreen(true));
    }
}

fn update_ending_badge(
    ending: Res<GameEnding>,
    badges: Query<&mut Visibility, With<GameEndingBadgeNode>>,
    texts: Query<&mut Text, With<GameEndingMonthText>>,
) {
    for mut visibility in badges {
        *visibility = match ending.month.is_some() {
            true => Visibility::Inherited,
            false => Visibility::Hidden,
        };
    }

    for mut text in texts {
        if let Some(month) = ending.month {
            text.0 = month.to_string();
        }
    }
}
