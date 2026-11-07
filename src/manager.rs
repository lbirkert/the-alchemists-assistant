use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.insert_resource(GameManager::default());
    app.add_systems(ON_GAME_ENTER, reset_game_manager);
    app.add_systems(GameUpdate, update_time);
}

pub const MAX_RECIPES: usize = 3;

/// Seconds per day
pub const DAY_DURATION: u64 = 60 * 3;

/// The game hour that starts a new day.
pub const DAY_START_HOUR: u8 = 6;

/// The game hour that ends a day.
pub const DAY_END_HOUR: u8 = 20;

/// The day of the month at which the game starts. This is a Monday.
const FIRST_DAY: u8 = 0;

/// February
const FIRST_MONTH: u8 = 1;

pub const FIRST_INGAME_DAY: u32 = (FIRST_DAY + FIRST_MONTH * 28) as _;

const WEEK_DAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[derive(Resource, Reflect, Default, Clone)]
#[reflect(Resource)]
pub struct GameManager {
    /// Current day number in game time.
    pub day: u32,
    /// Number of days actually played (starts at zero, advances after each day)
    pub days_played: u32,
    /// A looping timer tracking the current days time
    pub day_time: Timer,
    pub day_state: DayState,
    /// current active recipes, should always be #MAX_RECIPES
    pub recipes: Vec<Recipe>,

    /// score variable
    pub score: u32,
}

#[derive(Reflect, Default, Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn game_day_of_month(&self) -> u8 {
        (self.day % 28 + 1) as _
    }

    pub fn game_week_day(&self) -> &'static str {
        WEEK_DAYS[(self.day % 28 % 7) as usize]
    }

    pub fn game_month(&self) -> &'static str {
        let month = (self.day / 28) as usize;
        MONTHS[month % 12]
    }

    fn advance_day(&mut self) {
        self.day += 1;
        let week_day = self.day % 7;
        if week_day > 5 {
            self.day += 2;
        }
    }
}

pub fn reset_game_manager(mut manager: Single<&mut GameManager>) {
    let GameManager {
        day,
        days_played,
        day_time,
        day_state,
        recipes,
        score,
    } = &mut **manager;
    *day = FIRST_INGAME_DAY;
    *days_played = 0;
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
        manager.advance_day();
        info!("new day: {}", manager.day);
    }
}
