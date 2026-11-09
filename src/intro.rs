use bevy_easings::EaseMethod;

use crate::{prelude::*, utils::color_overlay::ColorOverlay};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameIntro>();

    app.add_systems(
        OnEnter(CutScene::None),
        set_state_cut_scene.run_if(in_state(GameScene::Intro)),
    );
    app.add_systems(OnExit(GameScene::Intro), set_state_cut_scene_none);

    app.add_systems(GameUpdate, cycle_textures);
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct GameIntro {
    pub finished: bool,
    pub state: ComputerState,
}

#[derive(Reflect, Default, Clone)]
pub enum ComputerState {
    #[default]
    News,
    FindJob,
    Apply,
    Finish {
        timer: Timer,
    },
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Interactable::IntroComputer)]
#[type_path = "scene"]
struct ComputerScreen;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Interactable::IntroComputer)]
#[type_path = "scene"]
struct ComputerMouse;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add)]
#[type_path = "scene"]
struct IntroNoise;

impl IntroNoise {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().spawn_scene(bsn! {
            ChildOf({ctx.entity})
            Transform
            AudioPlayer("audio/intro/intro_noise.ogg")
            PlaybackSettings::LOOP.with_spatial(true)
        });
    }
}

fn set_state_cut_scene(mut cut_scene: ResMut<NextState<CutScene>>) {
    cut_scene.set(CutScene::Intro);
}

fn set_state_cut_scene_none(mut cut_scene: ResMut<NextState<CutScene>>) {
    cut_scene.set(CutScene::None);
}

fn cycle_textures(
    mut messages: MessageReader<PlayerInteraction>,
    time: Res<Time>,
    mut commands: Commands,
    mut intro: ResMut<GameIntro>,
    mut scene: ResMut<NextState<GameScene>>,
    asset_server: Res<AssetServer>,
    query: Query<&MeshMaterial3d<StandardMaterial>, With<ComputerScreen>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mouse: Option<Single<Entity, With<ComputerMouse>>>,
    overlay: Single<Entity, With<ColorOverlay>>,
) {
    if let ComputerState::Finish { timer } = &mut intro.state {
        timer.tick(time.delta());
        if timer.just_finished() {
            scene.set(GameScene::Game);
        }
        return;
    }

    let Some(PlayerInteraction {
        entity,
        kind: Interactable::IntroComputer,
        button,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if let Some(mouse) = mouse {
        commands.spawn_scene(audio_scene(*mouse, &["audio/intro/mouse_click.ogg"]));
    }

    intro.state = match (&intro.state, button) {
        (ComputerState::News, MouseButton::Right) => ComputerState::FindJob,
        (ComputerState::FindJob, MouseButton::Right) => ComputerState::Apply,
        (ComputerState::FindJob, MouseButton::Left) => ComputerState::News,
        (ComputerState::Apply, MouseButton::Left) => ComputerState::FindJob,
        (ComputerState::Apply, MouseButton::Right) => {
            intro.finished = true;
            commands.entity(*overlay).insert(
                BackgroundColor(Color::NONE)
                    .ease_to(
                        BackgroundColor(Color::BLACK),
                        EaseFunction::CubicIn,
                        EasingType::Once {
                            duration: Duration::from_secs(1),
                        },
                    )
                    .ease_to(
                        BackgroundColor(Color::BLACK),
                        EaseMethod::Linear,
                        EasingType::Once {
                            duration: Duration::from_secs(2),
                        },
                    )
                    .ease_to(
                        BackgroundColor(Color::NONE),
                        EaseFunction::CubicOut,
                        EasingType::Once {
                            duration: Duration::from_secs(2),
                        },
                    ),
            );
            ComputerState::Finish {
                timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
            }
        }
        _ => return,
    };

    if let Ok(mat) = query.get(*entity)
        && let Some(mut mat) = mats.get_mut(&mat.0)
    {
        let path = match intro.state {
            ComputerState::News => "IntroNews.png",
            ComputerState::FindJob => "IntroJobPortal.png",
            ComputerState::Apply | ComputerState::Finish { .. } => "IntroApply.png",
        };
        let handle = asset_server.load(path);
        mat.base_color_texture = Some(handle.clone());
        mat.emissive_texture = Some(handle);
    }
}
