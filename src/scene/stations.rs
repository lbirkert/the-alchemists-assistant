use bevy::world_serialization::WorldInstanceReady;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, (station_interaction, update_station));
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Interactable::Station)]
#[component(on_add)]
pub struct ProcessingStation {
    pub state: StationState,
    pub kind: Station,
    set_animation: bool,
}

#[derive(Reflect, EnumIter, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Station {
    #[default]
    Mortar,
    CuttingBoard,
    WaterTap,
}

#[derive(Reflect, Default)]
pub enum StationState {
    #[default]
    Idle,
    Processing {
        working: bool,
        item: Item,
        item_entity: Entity,
        timer: Timer,
    },
    Done {
        item: Item,
        item_entity: Entity,
    },
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
struct StationArmature;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ArmatureHandle(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct StationAnimations {
    animations: Vec<AnimationNodeIndex>,
}

impl ProcessingStation {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let Some(this) = world.get::<Self>(ctx.entity) else {
            return;
        };
        let path = format!("{}#Scene0", this.asset_path());
        world.commands().entity(ctx.entity).apply_scene(bsn! {
            WorldAssetRoot(path)
            on(instance_ready_setup_armature)
        });
    }

    const fn num_animations(&self) -> usize {
        match self.kind {
            Station::Mortar => 2,
            Station::CuttingBoard => 2,
            Station::WaterTap => unimplemented!(),
        }
    }

    const fn asset_path(&self) -> &'static str {
        match self.kind {
            Station::Mortar => "Mortar.glb",
            Station::CuttingBoard => "CuttingBoard.glb",
            Station::WaterTap => unimplemented!(),
        }
    }

    pub const fn base_verb(&self, capitalize: bool) -> &'static str {
        match self.kind {
            Station::Mortar if capitalize => "Pestle",
            Station::Mortar => "pestle",
            Station::CuttingBoard if capitalize => "Cut",
            Station::CuttingBoard => "cut",
            Station::WaterTap if capitalize => "Wash",
            Station::WaterTap => "wash",
        }
    }

    pub const fn ing_verb(&self) -> &'static str {
        match self.kind {
            Station::Mortar => "pestling",
            Station::CuttingBoard => "cutting",
            Station::WaterTap => "washing",
        }
    }

    fn get_idle_animation(&self, handle: &StationAnimations) -> Option<AnimationNodeIndex> {
        match self.kind {
            Station::Mortar => handle.animations.first(),
            Station::CuttingBoard => handle.animations.get(1),
            Station::WaterTap => unimplemented!(),
        }
        .copied()
    }

    fn get_processing_animation(&self, handle: &StationAnimations) -> Option<AnimationNodeIndex> {
        match self.kind {
            Station::Mortar => handle.animations.get(1),
            Station::CuttingBoard => handle.animations.first(),
            Station::WaterTap => unimplemented!(),
        }
        .copied()
    }
}

impl Station {
    pub fn process_item(&self, item: Item) -> Item {
        match self {
            Station::Mortar => self.pestle_item(item),
            Station::CuttingBoard => self.cut_item(item),
            Station::WaterTap => unimplemented!(),
        }
    }

    fn pestle_item(&self, item: Item) -> Item {
        match item {
            Item {
                kind: ItemKind::Garlic,
                state: ItemState::Unprocessed,
                ..
            } => item.grinded(),
            Item {
                kind: ItemKind::Garlic,
                state: ItemState::Sliced | ItemState::Grinded,
                ..
            } => item.fine_grinded(),
            _ => item,
        }
    }

    fn cut_item(&self, item: Item) -> Item {
        match item {
            Item {
                kind: ItemKind::Garlic,
                state: ItemState::Unprocessed,
                ..
            } => item.sliced(),
            _ => item,
        }
    }
}

fn instance_ready_setup_armature(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    armatures: Query<Entity, With<StationArmature>>,
    search_stations: Query<(Option<&ChildOf>, Has<ProcessingStation>)>,
    stations: Query<&ProcessingStation>,
) {
    for armature_entity in armatures {
        if let Some(station_entity) = crate::utils::find_parent(armature_entity, &search_stations)
            && station_entity == event.entity
            && let Ok(station) = stations.get(station_entity)
        {
            let (graph, animations) = AnimationGraph::from_clips(
                (0..station.num_animations())
                    .map(|i| asset_server.load(format!("{}#Animation{i}", station.asset_path()))),
            );
            let graph_handle = graphs.add(graph);

            commands.entity(armature_entity).insert((
                AnimationGraphHandle(graph_handle),
                AnimationTransitions::new(),
                StationAnimations { animations },
            ));
            commands
                .entity(event.entity)
                .insert(ArmatureHandle(armature_entity));
            return;
        }
    }
    error!(
        "Failed to find station armature entity for station {}",
        event.entity
    );
}

fn station_interaction(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut inventory: Single<&mut Inventory>,
    mut messages: MessageReader<PlayerInteraction>,
    mut stations: Query<&mut ProcessingStation>,
) {
    if let Some(PlayerInteraction {
        entity,
        kind: Interactable::Station,
        ..
    }) = messages.read().next()
        && let Ok(mut station) = stations.get_mut(*entity)
    {
        match (&station.state, inventory.selection.as_ref()) {
            (StationState::Idle, Some(Selection::Item(item))) => {
                let scene = item.clone().scene(
                    &asset_server,
                    Transform::from_translation(Vec3::new(0., 0.2, 0.)),
                );
                let item_entity = commands
                    .spawn_scene(bsn! {
                        #StationItem
                        ChildOf({*entity})
                        @scene
                    })
                    .id();
                station.state = StationState::Processing {
                    working: true,
                    item: item.clone(),
                    item_entity,
                    timer: Timer::new(Duration::from_millis(2000), TimerMode::Once),
                };
                station.set_animation = true;
                inventory.selection = None;
            }
            (StationState::Processing { .. }, _) => {
                station.set_animation = true;
            }
            (StationState::Done { item, item_entity }, None) => {
                commands.entity(*item_entity).despawn();
                inventory.selection =
                    Some(Selection::Item(station.kind.process_item(item.clone())));
                station.state = StationState::Idle;
                station.set_animation = true;
            }
            _ => {}
        }
    }
}

fn update_station(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    focus: Res<PlayerFocus>,
    input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut statistics: ResMut<Statistics>,
    stations: Query<(Entity, &ArmatureHandle, &mut ProcessingStation)>,
    mut station_armatures: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &StationAnimations,
    )>,
) {
    for (entity, ArmatureHandle(armature_entity), mut station) in stations {
        if station.set_animation
            && let Ok((mut animation_player, mut transitions, animations)) =
                station_armatures.get_mut(*armature_entity)
        {
            station.set_animation = false;
            let animation = match station.state {
                StationState::Processing { working: true, .. } => {
                    station.get_processing_animation(animations)
                }
                _ => station.get_idle_animation(animations),
            };
            if let Some(animation) = animation {
                transitions
                    .play(&mut animation_player, animation, Duration::from_millis(100))
                    .repeat();
            }
        }

        let station_kind = station.kind;
        match &mut station.state {
            StationState::Idle => {}
            StationState::Processing {
                working,
                item,
                item_entity,
                timer,
            } => {
                if input.pressed(MouseButton::Right)
                    && focus.interactable == Some((entity, Interactable::Station))
                {
                    *working = true;
                    timer.tick(time.delta());
                    if timer.just_finished() {
                        let output_item = station_kind.process_item(item.clone());
                        commands.entity(*item_entity).despawn();
                        let scene = output_item.scene(
                            &asset_server,
                            Transform::from_translation(Vec3::new(0., 0.2, 0.)),
                        );
                        let new_item_entity = commands
                            .spawn_scene(bsn! {
                                #StationItem
                                ChildOf(entity)
                                @scene
                            })
                            .id();
                        statistics.process_item(item.clone());
                        station.state = StationState::Done {
                            item: item.clone(),
                            item_entity: new_item_entity,
                        };
                        station.set_animation = true;
                    }
                } else {
                    *working = false;
                    station.set_animation = true;
                }
            }
            StationState::Done { .. } => {}
        }
    }
}
