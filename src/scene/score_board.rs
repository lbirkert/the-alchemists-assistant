use crate::{manager::WEEK_DAYS, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, update_score_board_text);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add)]
#[type_path = "the_alchemist_assistant::scene::score_board"]
struct ScoreBoard;

#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
struct ScoreBoardCell {
    row: u8,
    column: u8,
}

impl ScoreBoard {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let width = 676;
        let height = 512;

        // This is the texture that will be rendered to.
        let image = Image::new_target_texture(
            width,
            height,
            bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
            None,
        );

        let mut images = world.resource_mut::<Assets<Image>>();
        let image_handle = images.add(image);

        let texture_camera = world
            .commands()
            .spawn((
                Name::new("ScoreBoard Text Camera"),
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

        let node = world
            .commands()
            .spawn_scene(bsn! {
                #ScoreBoardTextNode
                Node {
                    width: percent(100),
                    height: percent(100),
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex::<Vec<_>>(5, 1.0),
                    // Set the grid to have 5 rows all with sizes minmax(0, 1fr)
                    // This creates 5 exactly evenly sized rows
                    grid_template_rows: RepeatedGridTrack::flex::<Vec<_>>(5, 1.0),
                }
                UiTargetCamera(texture_camera)
                DespawnOnExit::<GameState>(GameState::Game)
            })
            .id();

        for row in 0..5 {
            for column in 0..5 {
                let cell = world
                    .commands()
                    .spawn_scene(bsn! {
                        #ScoreBoardCell
                        ChildOf(node)
                        ScoreBoardCell {
                            row,
                            column
                        }
                        Node {
                            width: percent(100),
                            height: percent(100),
                            align_items: AlignItems::Center,
                            justify_items: JustifyItems::Center
                        }
                    })
                    .id();

                if row == 0 {
                    let label = &WEEK_DAYS[column as usize][..3];
                    world.commands().spawn_scene(text_scene(label, cell));
                }
            }
        }

        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let scale = 0.8;
        let mesh_handle = meshes.add(Plane3d::new(
            Vec3::NEG_Y,
            Vec2::new(scale, height as f32 / width as f32 * scale),
        ));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(image_handle),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.,
            unlit: false,
            ..default()
        });

        world.commands().spawn_scene(bsn! {
            #ScoreBoardTextPlane
            ChildOf({ctx.entity})
            Mesh3d(mesh_handle)
            MeshMaterial3d::<StandardMaterial>(material_handle)
            NotShadowCaster
            Transform::from_xyz(0., -0.01, 0.)
        });
    }
}

fn text_scene<S: Into<String> + Send + Sync + 'static>(text: S, parent: Entity) -> impl Scene {
    bsn! {
        #ScoreBoardTextCell
        Text::new(text)
        TextFont {
            font_size: px(40),
            weight: FontWeight::EXTRA_BOLD
        }
        Node {
            width: percent(100)
        }
        TextLayout {
            justify: Justify::Center
        }
        TextColor(Color::WHITE)
        ChildOf(parent)
    }
}

fn update_score_board_text(
    manager: Res<GameManager>,
    mut commands: Commands,
    mut next_day: Local<usize>,
    cells: Query<(Entity, &ScoreBoardCell)>,
) {
    // Only update when a new day has started.
    if manager.day < *next_day {
        return;
    }
    *next_day += 1;

    for (entity, cell) in cells {
        if cell.row > 4 || cell.column > 6 {
            error!("Unexpected cell {cell:?}");
            continue;
        } else if cell.row == 0 {
            continue;
        }

        commands.entity(entity).despawn_children();

        let day_of_month = (cell.row - 1) * 7 + cell.column;
        let game_day = manager.game_month() * 28 + day_of_month as usize;
        if manager.skip_day(game_day) {
            commands.spawn_scene(text_scene("/", entity));
        } else if let Some(scores) = manager.get_score(game_day)
            && manager.day != game_day
        {
            spawn_day_score(&mut commands, entity, scores);
        }

        if manager.game_day_of_month() + 6 == cell.row * 7 + cell.column {
            commands.spawn_scene(bsn! {
                #CurrentDayHighlight
                Node {
                    width: percent(100),
                    height: percent(100),
                    position_type: PositionType::Absolute,
                    margin: UiRect::all(px(5)),
                    border: UiRect::all(px(2))
                }
                BorderColor::all(Color::srgb(1., 0., 0.))
                ZIndex(-1)
                ChildOf(entity)
            });
        }
    }
}

fn spawn_day_score(commands: &mut Commands, parent: Entity, scores: crate::manager::Scores) {
    let mut cmds = commands.spawn_scene(bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(3)
        }
        ChildOf(parent)
    });

    if scores.potions_submitted == 0 {
        cmds.apply_scene(bsn! {
            Children [
                @score(0, "icons/potion_warn.png")
            ]
        });
    } else if scores.invalid_potions == 0 {
        cmds.apply_scene(bsn! {
            Children [
                @score(scores.potions_submitted, "icons/potion_success.png")
            ]
        });
    } else if scores.potions_submitted == scores.invalid_potions {
        cmds.apply_scene(bsn! {
            Children [
                @score(scores.invalid_potions, "icons/potion_error.png")
            ]
        });
    } else {
        let valid_potions = scores.potions_submitted - scores.invalid_potions;
        cmds.apply_scene(bsn! {
            Children [
                @score(valid_potions, "icons/potion_success.png")
                --
                #Spacing
                Node {
                    width: px(5)
                }
                --
                @score(scores.invalid_potions, "icons/potion_error.png")
            ]
        });
    }
}

fn score(count: impl ToString, path: &'static str) -> impl Scene {
    let count = count.to_string();
    bsn! {
        Node {
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(3)
        }
        Children [
            Text::new(count)
            TextColor(Color::WHITE)
            TextFont {
                font_size: px(28),
            }
            --
            ImageNode {
                image: path
            }
            Node {
                width: px(24),
                height: px(36),
                margin: UiRect {
                    top: px(-4)
                }
            }
        ]
    }
}
