// This file defines all the data structures that are shared
// between your `backend` server and your `frontend` UI.
// This is the "common language" they both speak.

pub mod expert;
pub mod reflection;

#[cfg(feature = "ssr")]
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet}; // For loading static data

// --- Data Structures from quests.py ---
// These are direct Rust translations of your Python quest data.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QuestReward {
    #[serde(rename = "type")]
    pub reward_type: String,
    // Using Option<> for fields that aren't always present
    #[serde(default)] // Use default (None) if key is missing
    pub value: Option<i32>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub name: Option<String>, // For items
    #[serde(default)]
    pub target: Option<String>, // For relationships
    #[serde(default)]
    pub change: Option<i32>, // For relationships
    #[serde(default)]
    pub set_flag: Option<HashMap<String, bool>>, // For info type
    #[serde(default)]
    pub silent: Option<bool>,
}

// --- Persona Engine Data Structures ---

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Archetype {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stat {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ArchetypeStatBuff {
    pub archetype_id: i32,
    pub stat_id: i32,
    pub buff_value: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Dilemma {
    pub id: i32,
    pub title: String,
    pub dilemma_text: String,
    pub choices: Vec<DilemmaChoice>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DilemmaChoice {
    pub id: i32,
    pub dilemma_id: i32,
    pub choice_text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DilemmaChoiceArchetypePoint {
    pub dilemma_choice_id: i32,
    pub archetype_id: i32,
    pub points: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QuizSubmission {
    pub answers: HashMap<i32, i32>, // dilemma_id -> choice_id
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Choice {
    pub text: String,
    pub command: String,
    pub next_step: String,
    #[serde(default)]
    pub required_archetype_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestStep {
    pub description: String,
    #[serde(default)]
    pub choices: Vec<Choice>,
    pub trigger_condition: String,
    pub next_step: Option<String>,
    pub step_reward: Option<QuestReward>,
    #[serde(default)] // Default to `false` if missing
    pub is_major_plot_point: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Quest {
    pub title: String,
    pub chapter_theme: String,
    pub description: String,
    pub starting_step: String,
    pub completion_reward: QuestReward,
    pub steps: HashMap<String, QuestStep>,
}

// Type alias for our main quest data map
pub type QuestData = HashMap<String, Quest>;

// --- Data Structure from premade_characters.json ---
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CharacterTemplate {
    pub id: String,
    pub name: String,
    pub race_name: String,
    pub class_name: String,
    pub philosophy_name: String,
    pub boon: String,
    pub backstory: String,
    pub starting_quest_id: String,
    pub display_desc: String,
}

// --- Data Structures from character.py (RACE_DATA, etc.) ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RaceData {
    pub abilities: Vec<String>,
    pub fate_point_mod: i32,
}
// (We would also add ClassData and PhilosophyData here)

// --- Main PlayerCharacter Struct ---
// This is the "master" struct for a player's character,
// combining all the data from your Python app's `load_character_data`.
#[cfg(feature = "ssr")]
use bevy_ecs::prelude::Component;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Component))]
pub struct PlayerCharacter {
    pub id: String, // The character's document ID
    pub user_id: String,
    pub name: String,
    pub race_name: String,
    pub class_name: String,
    pub philosophy_name: String,
    pub boon: String,
    pub backstory: String,
    pub abilities: Vec<String>,
    pub aspects: Vec<String>,
    pub inventory: Vec<String>,             // From FS_INVENTORY
    pub quest_flags: HashMap<String, bool>, // From FS_QUEST_FLAGS
    pub current_location: String,
    pub current_quest_id: Option<String>,
    pub current_step_id: Option<String>,
    pub current_quest_title: String,
    pub current_step_description: String,
    pub fate_points: i32,
    pub report_summaries: Vec<ReportSummary>,

    // --- Persona Engine Fields ---
    #[serde(default)]
    pub primary_archetype_id: Option<i32>,
    #[serde(default)]
    pub stats: HashMap<String, i32>,

    // --- Fields managed ONLY by the server ---
    // We `skip` serializing them when sending to the frontend
    // to save bandwidth and keep secrets (if any).
    #[serde(skip)]
    pub learned_vocab: HashSet<String>,
}

// --- Structs for Profile Page (`profile.html`) ---
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProfileData {
    pub email: String,
    pub has_premium: bool,
    pub characters: Vec<CharacterSummary>,
    pub premade_characters: Vec<CharacterTemplate>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CharacterSummary {
    pub id: String,
    pub name: String,
    pub race: String,
    pub class_name: String,
}

// --- Structs for Journal Page (`journal_vocab_report.html`) ---
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JournalData {
    pub awl_words: Vec<VocabEntry>,
    pub ai_word_lists: HashMap<String, Vec<VocabEntry>>,
    pub report_summaries: Vec<ReportSummary>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VocabEntry {
    pub word: String,
    pub definition: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReportSummary {
    pub chapter: u32,
    pub summary: String,
    pub comprehension_score: f32,
    pub player_xp_gained: i32,
}

// --- (IMPROVEMENT) Structs for Game Interactivity ---
// These are new! They define the "contract" for
// submitting a command and getting a response.

/// This is the data the frontend sends to the backend.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerCommand {
    pub command_text: String,
    // We send the *entire* current character state
    // so the backend can modify it and send it back.
    pub current_character: PlayerCharacter,
}

/// This is the data the backend sends back to the frontend
/// after processing a command.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerProfile {
    pub id: i32,
    pub username: String,
    pub archetype: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameTurn {
    pub player_command: String,             // The command the player sent
    pub ai_narrative: String,               // The AI's response
    pub system_message: Option<String>,     // e.g., "Quest Completed!"
    pub updated_character: PlayerCharacter, // The *new* state
}

// --- Load Static Game Data ---
// This Rust pattern is the equivalent of your Python module-level dictionaries.
// It loads the data from the JSON files *at compile time* and parses them
// *once* when the application first runs.

// `cfg(feature = "ssr")` means "only include this code when compiling for the server"
// This keeps the large JSON data out of the frontend Wasm file.
#[cfg(feature = "ssr")]
pub static QUEST_DATA: Lazy<QuestData> = Lazy::new(|| {
    let quest_json = include_str!("quests.json");
    serde_json::from_str(quest_json).expect("Failed to parse quests.json")
});

#[cfg(feature = "ssr")]
pub static CHARACTER_TEMPLATES: Lazy<Vec<CharacterTemplate>> = Lazy::new(|| {
    let char_json = include_str!("characters.json");
    serde_json::from_str(char_json).expect("Failed to parse characters.json")
});

#[cfg(feature = "ssr")]
pub static RACE_DATA_MAP: Lazy<HashMap<String, RaceData>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "Sasquatch".to_string(),
        RaceData {
            abilities: vec!["Natural Armor".to_string(), "Cannot Wear Armor".to_string()],
            fate_point_mod: 0,
        },
    );
    m.insert(
        "Leprechaun".to_string(),
        RaceData {
            abilities: vec!["Fortunate Find".to_string()],
            fate_point_mod: 0,
        },
    );
    m.insert(
        "Android".to_string(),
        RaceData {
            abilities: vec!["Integrated Systems".to_string(), "Memory Limit".to_string()],
            fate_point_mod: -1,
        },
    );
    m.insert(
        "Opossuman".to_string(),
        RaceData {
            abilities: vec!["Pack Tactics".to_string(), "Fierce Loyalty".to_string()],
            fate_point_mod: 0,
        },
    );
    m.insert(
        "Tortisian".to_string(),
        RaceData {
            abilities: vec!["Artistic Shell".to_string(), "Second Brain".to_string()],
            fate_point_mod: 0,
        },
    );
    m.insert(
        "Slime".to_string(),
        RaceData {
            abilities: vec!["Absorb Magic".to_string(), "Shapechange".to_string()],
            fate_point_mod: 0,
        },
    );
    m
});
