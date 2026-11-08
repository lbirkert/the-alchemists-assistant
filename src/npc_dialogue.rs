use crate::{potion::PotionReport, prelude::*};

pub struct NpcDialogue;

pub const FIRST_GREETING: &str = "Hello, I am the Alchemist. You must be Greg right?\nYour job is to brew potions for me.\nI will bring you the recipes, the rest is self explanitory.\n Now, get to work! We don't have all day.";

pub const POSITIVE_GREETINGS: [&str; 3] = [
    "Hi there Greg.\nYou did well yesterday.\nContinue like this and you will soon be employee of the month or something. - Who want's that though?\nAnyways, get to work!",
    "Greg my friend.\nYesterday was awesome. Hope today you will be just as great!\nAnyways, we don't got much time. Get to work!",
    "Human Greg.\nYou amaze me. Well done yesterday.\nToday is a busy day. Make sure to keep up your pacing!\nLet me bring you the recipes...",
];

pub const NEUTRAL_GREETINGS: [&str; 3] = [
    "Good Morning Greg. Did you sleep well?\nI don't care actually. Get to work, today is a busy day!\nLet me bring you the recipes...",
    "Howdy My Friend.\nFeeling Good huh...?\nI also used to feel good when I was younger. Time runs quick.\nAnyways, I've prepared some recipes for you. Get to work!",
    "Hello Greg.\nI hope your morning was better than mine...\nNo?\nWell then some distraction is probably good. Get to work!",
];

pub const NEGATIVE_GREETINGS: [&str; 3] = [
    "Gregory we need to talk.\nI know that partying and drinking all day is fun and shit\nbut I need you to be sober at work. Yesterday wasn't your finest performance!\nGet to work!",
    "Greg.\nWhat. Did. You. Do. Yesterday?\nI feel like a disapointed dad that just watched his son poop into the sink.\nYou can do better! Get to work.",
    "We all have our days Greg, but I need you to improve your potion mixing skills. - Yesterday wasn't great.\n Get to work, we don't have time!",
];

pub const END_DAY_TEXTS: [&str; 6] = [
    "It is late Greg.\nTime to go home.",
    "Your shift is over.\nGo home now!",
    "No more time left Greg.\nYou should go home and sleep now!",
    "I like your ambition Greg,\nbut it's time to go home now!",
    "Time to go home, Greg.\nBe quick so I can lock the doors.",
    "I am tired.\nTime's over, go home now.",
];

impl NpcDialogue {
    pub fn generate_greeting(manager: &GameManager) -> impl ToString {
        if manager.day == 0 {
            return FIRST_GREETING;
        }

        let scores = manager.scores[manager.day - 1];
        // good potion => +2
        // bad potion => +1
        let score = 2 * scores.potions_submitted as i32 - scores.invalid_potions as i32;

        let mut rng = rand::rng();

        if score >= 7 {
            return POSITIVE_GREETINGS.choose(&mut rng).unwrap();
        }

        if score >= 4 {
            return NEUTRAL_GREETINGS.choose(&mut rng).unwrap();
        }

        NEGATIVE_GREETINGS.choose(&mut rng).unwrap()
    }

    pub fn generate_farewell(manager: &GameManager) -> impl ToString {
        let mut rng = rand::rng();
        END_DAY_TEXTS.choose(&mut rng).unwrap()
    }

    pub fn generate_farewell_last_of_month(manager: &GameManager) -> impl ToString {
        // let (total_scores, first_scores, last_scores) = manager.get_current_month_scores();
        // info!(
        //     "last day! scores:\n\
        //         total_scores: {total_scores:?}\n\
        //         first_scores: {first_scores:?}\n\
        //         last_scores: {last_scores:?}"
        // );
        // let message = if total_scores.potions_submitted == 0 {
        //     "What have you even done?"
        // } else {
        //     ""
        // };
        format!(
            "Well done Greg. This was the last day in {}.",
            manager.game_month_str(),
        )
    }

    pub fn generate_potion_report(report: &PotionReort, recipe: &Recipe) -> Option<String> {
        // TODO: we should also later add why the potion was good/wrong
        Some(match report {
            PotionReport::Valid => format!("{}", recipe.positive_result),
            PotionReport::WrongOrder => format!("{}", recipe.negative_result),
            PotionReport::Invalid { .. } => format!("{}", recipe.negative_result),
        })
    }
}
