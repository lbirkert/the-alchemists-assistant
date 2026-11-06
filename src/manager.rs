use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.insert_resource(GameManager::default());
    app.add_systems(ON_GAME_ENTER, reset_game_manager);
    app.add_systems(GameUpdate, update_time);
}

pub const MAX_RECIPES: usize = 3;
pub const DAY_DURATION: u64 = 60 * 3; // Seconds per day

pub const DAY_START_HOUR: u8 = 6; // The game hour that starts a new day.
pub const DAY_END_HOUR: u8 = 20; // The game hour that ends a day.

#[derive(Resource, Default, Clone)]
pub struct GameManager {
    pub day: u32,        // current day number (starts at zero)
    pub day_time: Timer, // A looping timer tracking the current days time
    pub day_state: DayState,
    pub recipes: Vec<Recipe>, // current active recipes, should always be #MAX_RECIPES

    pub score: u32, // score variable
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayState {
    #[default]
    /// The day has just started, the player has not been greeted yet.
    Started,
    Running,
    /// The day has ended (or not started yet)
    Ended,
}

impl GameManager {
    pub fn should_start_day(&self) -> bool {
        (DAY_START_HOUR..DAY_END_HOUR).contains(&self.game_hour())
    }

    pub fn should_end_day(&self) -> bool {
        self.game_hour() >= DAY_END_HOUR
    }

    pub fn should_bring_recipe(&self) -> bool {
        self.recipes.len() < MAX_RECIPES
    }

    /// Get the current game hour (from 0 am to 11 pm).
    pub fn game_hour(&self) -> u8 {
        (self.day_time.fraction() * 24.).floor() as _
    }
}

pub fn reset_game_manager(mut manager: Single<&mut GameManager>) {
    let GameManager {
        day,
        day_time,
        day_state,
        recipes,
        score,
    } = &mut **manager;
    *day = 0;
    *day_time = Timer::new(Duration::from_secs(DAY_DURATION), TimerMode::Repeating);
    day_time.set_elapsed(Duration::from_secs_f32(
        DAY_DURATION as f32 * DAY_START_HOUR as f32 / 24.,
    ));
    *day_state = DayState::Started;
    recipes.clear();
    *score = 0;
}

pub fn update_time(mut manager: Single<&mut GameManager>, time: Res<Time>) {
    manager.day_time.tick(time.delta());

    if manager.day_time.just_finished() {
        manager.day += 1;
        info!("new day: {}", manager.day);
    }
}
