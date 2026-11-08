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

pub const FIRST_INGAME_DAY: usize = (FIRST_DAY + FIRST_MONTH * 28) as _;

pub const WEEK_DAYS: [&str; 7] = [
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
    pub day: usize,
    /// Number of days actually played (starts at zero, advances after each day)
    pub days_played: usize,
    /// A looping timer tracking the current days time
    pub day_time: Timer,
    pub day_state: DayState,
    /// current active recipes, should always be #MAX_RECIPES
    pub recipes: Vec<Recipe>,

    /// The scores of each played day. This list is indexable by `days_played`.
    scores: Vec<Scores>,
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

#[derive(Reflect, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scores {
    /// The amount of potions submitted.
    pub potions_submitted: usize,
    /// The amount of submitted potions that did not conform to their recipe.
    /// This is a subset of `potions_submitted`.
    pub invalid_potions: usize,
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
        WEEK_DAYS[self.day % 28 % 7]
    }

    pub fn game_month(&self) -> usize {
        self.day / 28
    }

    pub fn game_month_str(&self) -> &'static str {
        MONTHS[self.game_month() % 12]
    }

    pub fn score_potion(&mut self, is_valid: bool) {
        self.scores[self.days_played].potions_submitted += 1;
        if !is_valid {
            self.scores[self.days_played].invalid_potions += 1;
        }
    }

    /// Let us ignore the algorithmic complexity here
    fn game_day_to_days_played(&self, game_day: usize) -> usize {
        let mut days_played = 0;
        for day in FIRST_INGAME_DAY..game_day {
            if !self.skip_day(day) {
                days_played += 1;
            }
        }
        days_played
    }

    pub fn get_score(&self, game_day: usize) -> Option<Scores> {
        let days_played = self.game_day_to_days_played(game_day);
        if days_played <= self.days_played {
            Some(self.scores[days_played])
        } else {
            None
        }
    }

    pub fn skip_day(&self, game_day: usize) -> bool {
        let week_day = game_day % 7;
        if week_day > 4 {
            return true;
        }

        let current_month = (game_day / 28) as u8;
        let current_week_of_month = game_day % 28 / 7;
        current_month == FIRST_MONTH && (1..=2).contains(&current_week_of_month)
    }

    fn advance_day(&mut self) {
        self.days_played += 1;
        self.scores.push(Scores::default());

        self.day += 1;
        while self.skip_day(self.day) {
            self.day += 1;
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
        scores,
    } = &mut **manager;
    *day = FIRST_INGAME_DAY;
    *days_played = 0;
    *day_time = Timer::new(Duration::from_secs(DAY_DURATION), TimerMode::Repeating);
    day_time.set_elapsed(Duration::from_secs_f32(
        DAY_DURATION as f32 * DAY_START_HOUR as f32 / 24.,
    ));
    *day_state = DayState::Started;
    recipes.clear();
    scores.clear();
    scores.push(Scores::default());
}

pub fn update_time(mut manager: Single<&mut GameManager>, time: Res<Time>) {
    manager.day_time.tick(time.delta());

    if manager.day_time.just_finished() {
        manager.advance_day();
        info!("new day: {}", manager.day);
    }
}
