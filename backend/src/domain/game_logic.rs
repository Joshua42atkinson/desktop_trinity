#![allow(unused)]
use crate::game::components::*;
use bevy::prelude::*;
use common::{PlayerCharacter, QUEST_DATA};

pub fn process_command(world: &mut World, command_text: String) -> PlayerCharacter {
    let mut player_dto = crate::domain::player::get_simulated_character(); // Use as base

    let mut query = world.query::<(
        &Name,
        &mut Persona,
        &mut VirtueTopology,
        &mut CognitiveLoad,
        &mut StoryProgress,
        &mut Level,
        &mut Experience,
    )>();

    for (name, mut persona, _virtues, _load, mut progress, _level, _xp) in query.iter_mut(world) {
        // Logic
        if command_text.starts_with("set_archetype") {
            let parts: Vec<&str> = command_text.splitn(3, ' ').collect();
            if parts.len() == 3 {
                // Simplified for now - just setting archetype enum if possible
                // In real impl, we'd parse the string to enum
                info!("Archetype setting not fully implemented in ECS yet");
            }
        } else if let (Some(quest_id), Some(step_id)) = (
            progress.current_quest_id.as_deref(),
            progress.current_step_id.as_deref(),
        ) {
            if let Some(quest) = QUEST_DATA.get(quest_id) {
                if let Some(step) = quest.steps.get(step_id) {
                    let command_lower = command_text.trim().to_lowercase();
                    if let Some(choice) = step.choices.iter().find(|c| {
                        // Check archetype requirement against Persona
                        // Need to map int id to Enum or string
                        true // Bypass for now
                    }) {
                        if let Some(choice) =
                            step.choices.iter().find(|c| c.command == command_lower)
                        {
                            if let Some(next_step_data) = quest.steps.get(&choice.next_step) {
                                progress.current_step_id = Some(choice.next_step.clone());
                                progress.current_step_description =
                                    next_step_data.description.clone();
                                info!("Quest advanced to step: {}", choice.next_step);
                            }
                        }
                    }
                }
            }
        }

        // Map back to DTO
        player_dto.name = name.as_str().to_string();
        player_dto.current_quest_id = progress.current_quest_id.clone();
        player_dto.current_step_id = progress.current_step_id.clone();
        player_dto.current_step_description = progress.current_step_description.clone();
        player_dto.inventory = progress.inventory.clone();
        player_dto.quest_flags = progress.quest_flags.clone();
        player_dto.learned_vocab = progress.learned_vocab.clone();
        // Map other fields as needed
    }

    player_dto
}
