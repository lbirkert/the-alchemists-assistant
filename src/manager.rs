use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.insert_resource(GameManager::default());
    app.add_systems(ON_GAME_ENTER, reset_game_manager);
    app.add_systems(GameUpdate, update_time);
}

pub const MAX_RECIPES: usize = 3;
pub const DAY_DURATION: u32 = 60 * 3; // 1 minutes
pub const DAY_GAME_DURATION: u32 = 3600; // 12 hours, e.g. 6:00 -> 18:00

#[derive(Resource, Default, Clone)]
pub struct GameManager {
    pub day: u32,              // current day number (starts at zero)
    pub day_start: u32,        // current day's starting time (users time)
    pub has_day_ended: bool,   // whether the player needs to leave now
    pub has_npc_greeted: bool, // whether the npc has greeted the player yet
    pub time: u32,             // current users time - day_start
    pub game_time: u32,        // current game time in minutes
    pub recipes: Vec<Recipe>,  // current active recipes, should always be #MAX_RECIPES

    pub score: u32, // score variable
}

impl GameManager {
    pub fn should_end_day(&self) -> bool {
        self.time > DAY_DURATION
    }

    pub fn should_bring_recipe(&self) -> bool {
        self.recipes.len() < MAX_RECIPES
    }

    pub fn should_greet(&self) -> bool {
        !self.has_npc_greeted
    }
}

pub fn reset_game_manager(mut manager: Single<&mut GameManager>, time: Res<Time>) {
    manager.day = 0;
    manager.day_start = time.elapsed_secs() as u32;
    manager.has_day_ended = false;
    manager.has_npc_greeted = false;
    manager.time = 0;
    manager.game_time = 0;
    manager.recipes.truncate(0);
    manager.score = 0;
}

pub fn update_time(mut manager: Single<&mut GameManager>, time: Res<Time>) {
    if manager.has_day_ended {
        return;
    }

    manager.time = time.elapsed_secs() as u32 - manager.day_start;
    let percentage_of_day = (manager.time as f32) / (DAY_DURATION as f32);
    manager.game_time = (percentage_of_day * DAY_GAME_DURATION as f32) as u32;
}
