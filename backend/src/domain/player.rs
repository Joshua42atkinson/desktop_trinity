use common::{
    PlayerCharacter,
    ReportSummary,
    CHARACTER_TEMPLATES, // Our static list of premade characters
    QUEST_DATA,          // Our static map of quest data
    RACE_DATA_MAP,       // Our static map of race data
};
use std::collections::{HashMap, HashSet};

// --- Simulated Database Function ---
// This creates a "Totem" character on-the-fly for testing.
// (Unchanged from previous version)
pub fn get_simulated_character() -> PlayerCharacter {
    let template = CHARACTER_TEMPLATES.first().cloned().unwrap();
    let base_fate_points = 1;
    let race_data = RACE_DATA_MAP.get(&template.race_name).unwrap();
    let mut quest_title = "No Quest".to_string();
    let mut step_desc = "You are ready for an adventure.".to_string();
    let start_quest_id_str = "Q_THE_WAY_IS_SHUT";
    let start_quest = QUEST_DATA.get(start_quest_id_str);

    let (start_step_id, start_quest_id) = if let Some(quest) = start_quest {
        quest_title = quest.title.clone();
        if let Some(step) = quest.steps.get(&quest.starting_step) {
            step_desc = step.description.clone();
        }
        (
            Some(quest.starting_step.clone()),
            Some(start_quest_id_str.to_string()),
        )
    } else {
        (None, None)
    };
    PlayerCharacter {
        id: "char_sim_totem_001".to_string(),
        user_id: "user_sim_001".to_string(),
        name: template.name,
        race_name: template.race_name,
        class_name: template.class_name,
        philosophy_name: template.philosophy_name,
        boon: template.boon,
        backstory: template.backstory,
        abilities: race_data.abilities.clone(),
        aspects: vec!["Weapon of Choice".to_string()],
        inventory: vec!["Rations".to_string(), "Trembling Porcupine".to_string()],
        quest_flags: HashMap::new(),
        current_location: "Thetopia - Town Square".to_string(),
        current_quest_id: start_quest_id,
        current_step_id: start_step_id,
        current_quest_title: quest_title,
        current_step_description: step_desc,
        fate_points: base_fate_points + race_data.fate_point_mod,
        learned_vocab: HashSet::new(),
        primary_archetype_id: None,
        stats: HashMap::new(),
        report_summaries: vec![ReportSummary {
            chapter: 1,
            summary:
                "**Chapter Complete!**\n* Comprehension Score: 8.5 / 10\n* Player XP Gained: +75"
                    .to_string(),
            comprehension_score: 8.5,
            player_xp_gained: 75,
        }],
    }
}
