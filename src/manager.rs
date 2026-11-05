use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, update_time);
}

pub const MAX_RECIPES: usize = 3;
pub const DAY_DURATION: u32 = 60 * 10; // 10 minutes
pub const DAY_GAME_DURATION: u32 = 3600; // 12 hours, e.g. 6:00 -> 18:00

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct GameManger {
    pub day: u32,               // current day number (starts at zero)
    pub day_start: Option<u32>, // current day's starting time (users time)
    pub day_ended: bool,        // whether the player needs to leave now
    pub time: u32,              // current users time - day_start
    pub game_time: u32,         // current game time in minutes
    pub recipes: Vec<Recipe>,   // current active recipes, should always be #MAX_RECIPES
}

pub fn update_time(manager: Single<&mut GameManager>, time: Res<Time>) {
    if manager.day_ended {
        return;
    }

    manager.time = time.elapsed_secs() as u32 - day_start;
    manager.game_time =
        ((manager.time as f32) / (DAY_DURATION as f32) * DAY_GAME_DURATION as f32) as u32;
}

// the npc is a state machine that reacts to GameManager and bell
//
// Idle: on new day -> GreetPlayer
// Idle: on bell pressed -> WalkToTable
// WalkToTable: on table has potion -> TakePotion(..)
// TakePotion: on walked back -> Idle
// WalkToTable: on table has no potion -> WalkBack + say something funny
// Idle: on # recipes < MAX_RECIPES -> BringRecipe
// Idle: on time > DAY_DURATION -> SayByeToPlayer
// SayByeToPlayer: on finished -> Idle + set day_ended to true
pub enum NpcState {
    Idle,
    GreetPlayer(bool), / true -> walk into room
    Walk(bool), // false -> walk to table, true -> walk back
    TakePotion(Potion),
    BringRecipe(Recipe),
    SayByeToPlayer(bool),
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct Npc {
    state: NpcState,
    target_pos: Vec3,
}

pub fn say_bye_to_player(manager: Single<&mut GameManger>, npc: Single<&mut Npc>) {
    if manager.day_ended {
        return;
    }

    if !matches!(npc.state.as_ref(), NpcState::Idle) || manager.time <= DAY_DURATION {
        return;
    }

    npc.state = NpcState::SayByeToPlayer;
}

pub fn update_npc(
    manager: Single<&mut GameManager>,
    npc_query: Single<(&mut Transform, &mut Npc)>,
    time: Res<Time>,
) {
    let (npc, transform) = npc_query.as_ref();

    match npc.state {
        NpcState::GreetPlayer | NpcState::WalkToTable | NpcState::WalkBack | NpcState::
    }
    if transform.translation != npc.target_position {
        let dt = time.delta().as_secs_f32();
        let movement = (npc.target_position - transform.translation).normalize() * dt;
        transform.translation += movement;
    }
}
