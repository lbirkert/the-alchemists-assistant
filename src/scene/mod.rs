use crate::prelude::*;

use bevy::light::NotShadowCaster;
use bevy_easings::EasingComponent;
use bevy_flora::{FloraPlugin, Growing, ProceduralTree, TreeAge, TreePreset};

pub mod stations;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(stations::plugin);
    app.add_plugins(FloraPlugin);

    app.add_systems(ON_GAME_ENTER, spawn_scene);
    app.add_systems(GameUpdate, (rotate_clock_hand, update_calendar_text));
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    RigidBody::Static,
    CollisionLayers::new(GameLayer::Environment, GameLayer::all_bits())
)]
pub struct EnvironmentObject;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
    EnvironmentObject
)]
struct ColliderFromMesh;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    Name::new("Sun Light"),
    PointLight {
        intensity: 400000.0,
        range: 400.0,
        radius: 400.0,
        color: Color::srgba(1.0, 1.0, 0.8, 1.0),
        shadow_maps_enabled: true,
        shadow_map_near_z: 0.4,
        contact_shadows_enabled: false,
        ..Default::default()
    },
)]
struct SunLight;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    Name::new("Cauldron Light"),
    PointLight {
        intensity: 80000.0,
        range: 5.0,
        radius: 5.0,
        color: Color::srgba(0.0, 1.0, 0.0, 1.0),
        shadow_maps_enabled: false,
        ..Default::default()
    },
)]
struct CauldronLight;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    Name::new("Tree"),
    ProceduralTree(TreePreset::Oak.params()),
    TreeAge(10.0),
    Growing {
        rate: 0.0,
    },
)]
struct Tree;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Interactable::Bell)]
pub struct NpcBell;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct NpcStickyNotes;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Interactable::NpcTable)]
pub struct NpcTable {
    pub potions: Vec<(Entity, PotionData)>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct GameClockHand;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add)]
struct CalendarBlock;

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
struct CalendarBlockWeekdayText;

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
struct CalendarBlockDayText;

impl NpcTable {
    pub fn validatable_potion(&mut self) -> Option<(Entity, PotionData)> {
        let idx = self.potions.iter().position(|(_, p)| p.can_validate())?;
        Some(self.potions.remove(idx))
    }
}

fn spawn_scene(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #GameScene
        WorldAssetRoot("TheAlchemistsAssistant.glb#Scene0")
        DespawnOnExit<GameState>(GameState::Game)
    });

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgba(1.0, 1.0, 0.8, 1.0),
        brightness: 100.0,
        ..default()
    });
}

fn rotate_clock_hand(
    manager: Res<GameManager>,
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<GameClockHand>, Without<EasingComponent<Transform>>)>,
) {
    let rotation = Quat::from_rotation_z(-TAU * manager.game_hour() as f32 / 24.);
    for (entity, transform) in query {
        if rotation.angle_between(transform.rotation) > 0.1 {
            commands.entity(entity).insert(transform.ease_to(
                transform.with_rotation(rotation),
                EaseFunction::ExponentialIn,
                EasingType::Once {
                    duration: Duration::from_millis(350),
                },
            ));
        }
    }
}

impl CalendarBlock {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
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
                DespawnOnExit(GameState::Game),
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
            DespawnOnExit::<GameState>(GameState::Game)
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
            &manager.game_month()[..3]
        );
    }
}
