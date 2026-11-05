// the npc is a state machine that reacts to game state changes
//
// Idle: on new day -> GreetPlayer
// Idle: on bell pressed -> WalkToTable
// WalkToTable: on table has potion -> TakePotion(..)
// TakePotion: on walked back -> Idle
// WalkToTable: on table has no potion -> WalkBack + say something funny
// Idle: on # recipes < MAX_RECIPES -> BringRecipe
// Idle: on time > DAY_DURATION -> SayByeToPlayer
// SayByeToPlayer: on finished -> Idle + set day_ended to true

use bevy::picking::backend::HitData;

use crate::{
    prelude::*,
    scene::{NpcStickyNotes, NpcTable},
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        GameUpdate,
        (
            update_walking_npc,
            update_idle_npc,
            update_speaking_npc,
            listen_for_bell,
        ),
    );
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct Npc {
    state_start: f32, // state starting time
    state: NpcState,
}

#[derive(Reflect, Clone, Default, Debug)]
pub enum NpcState {
    #[default]
    Idle,
    SpeakingToPlayer(MessageToPlayer),
    WalkIntoRoom(WalkIntoRoomReason),
    WalkOutOfRoom(WalkOutOfRoomReason),
}

#[derive(Reflect, Clone, Debug)]
pub enum MessageToPlayer {
    Bye(String),
    Hello(String),
}

#[derive(Reflect, Clone, Debug)]
pub enum WalkIntoRoomReason {
    BellPressed,
    BringRecipes(Vec<Recipe>),
    SayToPlayer(MessageToPlayer),
}

#[derive(Reflect, Clone, Debug)]
pub enum WalkOutOfRoomReason {
    SayToPlayer(MessageToPlayer),
    TakePotion(Potion),
    NoPotion,
}

fn update_idle_npc(
    mut manager: Single<&mut GameManager>,
    mut npc: Single<&mut Npc>,
    time: Res<Time>,
) {
    if manager.has_day_ended {
        // TODO: add a day running state maybe?
        return;
    }

    if !matches!(npc.state, NpcState::Idle) {
        return;
    }

    if manager.should_end_day() {
        npc.state_start = time.elapsed_secs();
        npc.state = NpcState::WalkIntoRoom(WalkIntoRoomReason::SayToPlayer(MessageToPlayer::Bye(
            "Bye Brotha".to_string(),
        )));
        // TODO: proper messages

        return;
    }

    if manager.should_greet() {
        manager.has_npc_greeted = true;
        npc.state_start = time.elapsed_secs();
        npc.state = NpcState::WalkIntoRoom(WalkIntoRoomReason::SayToPlayer(
            MessageToPlayer::Hello("Hello There!".to_string()),
        ));
        // TODO: proper messages

        return;
    }

    if manager.should_bring_recipe() {
        // collect all recipes at once so you don't have to walk so much
        let mut recipes = Vec::new();
        while manager.should_bring_recipe() {
            let recipe = Recipe::generate();
            manager.recipes.push(recipe.clone());
            recipes.push(recipe);
        }

        npc.state_start = time.elapsed_secs();
        npc.state = NpcState::WalkIntoRoom(WalkIntoRoomReason::BringRecipes(recipes));

        return;
    }
}

fn listen_for_bell(
    mut messages: MessageReader<PlayerInteraction>,
    mut npc: Single<&mut Npc>,
    manager: Single<&GameManager>,
    time: Res<Time>,
) {
    if manager.has_day_ended {
        // TODO: add a day running state maybe?
        return;
    }

    let Some(PlayerInteraction {
        kind: Interactable::Bell,
        hit: HitData { .. },
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !matches!(npc.state, NpcState::Idle) {
        return;
    }

    npc.state_start = time.elapsed_secs();
    npc.state = NpcState::WalkIntoRoom(WalkIntoRoomReason::BellPressed);
}

fn update_walking_npc(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut manager: Single<&mut GameManager>,
    mut npc_query: Single<(&mut Transform, &mut Npc)>,
    mut npc_table: Single<&mut NpcTable>,
    npc_sticky_notes: Single<Entity, With<NpcStickyNotes>>,
    time: Res<Time>,
) {
    if manager.has_day_ended {
        // TODO: add a day running state maybe?
        return;
    }

    let (ref mut transform, ref mut npc) = *npc_query;

    let target_pos = match npc.state {
        NpcState::WalkIntoRoom(WalkIntoRoomReason::SayToPlayer(..)) => Vec3::new(0.0, 0.0, 3.0),
        NpcState::WalkIntoRoom(..) => Vec3::new(0.0, 0.0, 10.0),
        NpcState::WalkOutOfRoom(..) => Vec3::new(0.0, 0.0, -5.0),
        _ => return,
    };

    if !walk_npc(transform, &time, target_pos) {
        return;
    }

    npc.state_start = time.elapsed_secs();
    npc.state = match npc.state {
        NpcState::WalkIntoRoom(ref reason) => match reason {
            WalkIntoRoomReason::BellPressed => match npc_table.validatable_potion() {
                Some((entity, potion)) => {
                    commands.entity(entity).despawn();
                    NpcState::WalkOutOfRoom(WalkOutOfRoomReason::TakePotion(potion))
                }
                None => NpcState::WalkOutOfRoom(WalkOutOfRoomReason::NoPotion),
            },
            WalkIntoRoomReason::BringRecipes(recipes) => {
                let mut offset = 0.0; // TODO: also account for existing sticky notes
                for recipe in recipes.iter() {
                    // spawn the sticky note
                    let scene = StickyNote {
                        text: recipe.task_description.clone(),
                        recipe: Some(recipe.clone()),
                    }
                    .scene(
                        &asset_server,
                        Vec3::new(0.0, offset, 0.0),
                        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    );
                    commands
                        .spawn_scene(scene)
                        .insert(ChildOf(*npc_sticky_notes));

                    offset += 0.02;
                }

                // check table for potions
                match npc_table.validatable_potion() {
                    Some((entity, potion)) => {
                        commands.entity(entity).despawn();
                        NpcState::WalkOutOfRoom(WalkOutOfRoomReason::TakePotion(potion))
                    }
                    None => NpcState::WalkOutOfRoom(WalkOutOfRoomReason::NoPotion),
                }
            }
            WalkIntoRoomReason::SayToPlayer(msg) => NpcState::SpeakingToPlayer(msg.clone()),
        },
        NpcState::WalkOutOfRoom(ref reason) => match reason {
            WalkOutOfRoomReason::NoPotion => NpcState::Idle,
            WalkOutOfRoomReason::TakePotion(_potion) => {
                todo!("process potion, update recipies");
            }
            WalkOutOfRoomReason::SayToPlayer(msg) => {
                if matches!(msg, MessageToPlayer::Bye(_)) {
                    manager.has_day_ended = true;
                }

                NpcState::Idle
            }
        },
        _ => panic!(),
    };
}

fn update_speaking_npc(manager: Single<&GameManager>, mut npc: Single<&mut Npc>, time: Res<Time>) {
    if manager.has_day_ended {
        // TODO: add a day running state maybe?
        return;
    }

    let NpcState::SpeakingToPlayer(ref msg) = npc.state else {
        return;
    };

    // TODO: how do we do the rendering?

    let duration = match msg {
        MessageToPlayer::Hello(_) => 5.0,
        MessageToPlayer::Bye(_) => 10.0,
    };

    let now = time.elapsed_secs();
    if now - npc.state_start <= duration {
        return;
    }

    npc.state = NpcState::WalkOutOfRoom(WalkOutOfRoomReason::SayToPlayer(msg.clone()));
    npc.state_start = now;
}

fn walk_npc(transform: &mut Transform, time: &Time, target_pos: Vec3) -> bool {
    const NPC_SPEED: f32 = 5.0;

    let delta = target_pos - transform.translation;
    if delta.length_squared() < 1e-3 {
        return true;
    }

    let dt = time.delta().as_secs_f32();
    let movement = delta.normalize() * dt * NPC_SPEED;
    transform.translation += movement;

    false
}
