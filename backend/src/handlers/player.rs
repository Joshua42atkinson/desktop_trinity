use crate::domain::player::get_simulated_character;
use crate::{AppState, Result};
use axum::{extract::State, Json};
use common::{
    CharacterSummary, GameTurn, JournalData, PlayerCharacter, PlayerCommand, PlayerProfile,
    ProfileData, VocabEntry, CHARACTER_TEMPLATES,
};
use std::collections::HashMap;

pub async fn handle_submit_command(
    State(_app_state): State<AppState>,
    Json(payload): Json<PlayerCommand>,
) -> Result<Json<GameTurn>> {
    // Mock implementation while AsyncWorld is disabled
    let updated_character = get_simulated_character();

    let game_turn = GameTurn {
        player_command: payload.command_text,
        ai_narrative: "Simulation mode: AsyncWorld disabled.".to_string(),
        system_message: None,
        updated_character,
    };

    Ok(Json(game_turn))
}

pub async fn get_player_character(
    State(_app_state): State<AppState>,
) -> Result<Json<PlayerCharacter>> {
    let character = get_simulated_character();
    Ok(Json(character))
}

pub async fn get_journal_data(State(_app_state): State<AppState>) -> Result<Json<JournalData>> {
    let character = get_simulated_character();
    let awl_words = vec![
        VocabEntry {
            word: "analyse".to_string(),
            definition: "To examine something methodically...".to_string(),
        },
        VocabEntry {
            word: "approach".to_string(),
            definition: "A way of dealing with something...".to_string(),
        },
    ];

    let mut ai_lists = HashMap::new();
    ai_lists.insert(
        "'Chaos' Context".to_string(),
        vec![VocabEntry {
            word: "entropy".to_string(),
            definition: "Lack of order or predictability...".to_string(),
        }],
    );

    let data = JournalData {
        awl_words,
        ai_word_lists: ai_lists,
        report_summaries: character.report_summaries,
    };
    Ok(Json(data))
}

pub async fn get_profile_data(State(_app_state): State<AppState>) -> Result<Json<ProfileData>> {
    let characters = vec![
        CharacterSummary {
            id: "char_sim_totem_001".to_string(),
            name: "Totem".to_string(),
            race: "Sasquatch".to_string(),
            class_name: "Soldier".to_string(),
        },
        CharacterSummary {
            id: "char_sim_bolt_002".to_string(),
            name: "Bolt".to_string(),
            race: "Android".to_string(),
            class_name: "Inventor".to_string(),
        },
    ];

    let data = ProfileData {
        email: "player@daydream.com".to_string(),
        has_premium: true,
        characters,
        premade_characters: CHARACTER_TEMPLATES.to_vec(),
    };
    Ok(Json(data))
}

pub async fn get_player_profile(State(_app_state): State<AppState>) -> Result<Json<PlayerProfile>> {
    // let player = sqlx::query_as!(PlayerProfile, "SELECT * FROM players WHERE id = $1", 1)
    //     .fetch_optional(&app_state.pool)
    //     .await?
    //     .ok_or(AppError::NotFound)?;

    Ok(Json(PlayerProfile {
        id: 1,
        username: "Daydreamer".to_string(),
        archetype: "The Sage".to_string(),
    }))
}
