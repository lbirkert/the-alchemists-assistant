use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, play_mortar_animation);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(EnvironmentObject, Collider::cuboid(2., 0.5, 0.8))]
struct AlchemyTable;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Interactable::Mortar)]
#[component(on_add)]
pub struct MortarStation {
    pub working: bool,
}

impl MortarStation {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().entity(ctx.entity).apply_scene(bsn! {
            WorldAssetRoot("Mortar.glb#Scene0")
        });
    }
}

fn play_mortar_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    assets_server: Res<AssetServer>,
    animation_players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut animation_player) in animation_players {
        let (graph, pestle) =
            AnimationGraph::from_clip(assets_server.load("Mortar.glb#Animation0"));
        let graph_handle = graphs.add(graph);
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(graph_handle));
        animation_player.play(pestle).repeat();
    }
}
