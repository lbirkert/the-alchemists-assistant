use crate::{potion::PotionReport, prelude::*};

pub struct NpcDialogue;

pub const GREETINGS: [&str; 3] = [
    "Good Morning Greg.\nI've already prepared everything.\nNow get to work!",
    "Howdy My Friend.\nFeeling Good huh...?\nYes?\nGet to work then!",
    "Hello Greg.\nI hope your morning was better than mine...\nNo?\nLife goes on Anyways, just get to work!",
];

// TODO: proper messages
impl NpcDialogue {
    pub fn generate_greeting(manager: &GameManager) -> impl ToString {
        let mut rng = rand::rng();
        GREETINGS.choose(&mut rng).unwrap()
    }

    pub fn generate_farewell(manager: &GameManager) -> impl ToString {
        "It is evening already.\n\
            I will go to sleep now.\n\
            You should too."
    }

    pub fn generate_farewell_last_of_month(manager: &GameManager) -> impl ToString {
        let (total_scores, first_scores, last_scores) = manager.get_current_month_scores();
        info!(
            "last day! scores:\n\
                total_scores: {total_scores:?}\n\
                first_scores: {first_scores:?}\n\
                last_scores: {last_scores:?}"
        );
        let message = if total_scores.potions_submitted == 0 {
            "What have you even done?"
        } else {
            ""
        };
        format!(
            "This is the last day in this {}.\n{message}",
            manager.game_month_str()
        )
    }

    pub fn generate_potion_report(
        manager: &GameManager,
        report: &PotionReport,
    ) -> Option<impl ToString> {
        Some(match report {
            PotionReport::Valid => return None,
            PotionReport::WrongOrder => {
                "Your potion seems correct but on closer inspection, something is off."
            }
            PotionReport::Invalid { .. } => "Invalid potion",
        })
    }
}
