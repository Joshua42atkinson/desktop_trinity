use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub link: Option<String>,   // URL to the document/video
    pub link_text: String,      // "Read Report", "Watch Video"
    pub icon: String,           // SVG string
}

// Add the following structs:
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerCommand {
    pub command_text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameTurn {
    pub player_command: String,
    pub ai_narrative: String,
    pub system_message: Option<String>,
    // pub updated_character: PlayerCharacter,
}