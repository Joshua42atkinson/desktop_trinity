use crate::game::components::*;
use bevy::prelude::*;
use bevy_yarnspinner::events::{DialogueCompleteEvent, ExecuteCommandEvent};

pub fn update_virtue_topology(
    mut query: Query<(&mut VirtueTopology, &StoryProgress), Changed<StoryProgress>>,
) {
    for (mut virtues, progress) in query.iter_mut() {
        // Logic: If the story has progressed, it implies a decision was made.
        // In a real implementation, we would look up the specific choice in a database
        // to determine which virtues to adjust.
        // For now, we simulate the "willingness to make a decision" (Self-Efficacy).

        if let Some(_step) = &progress.current_step_id {
            // Boost Self-Efficacy for taking action
            virtues.self_efficacy = (virtues.self_efficacy + 0.05).min(1.0);
            info!(
                "Virtue Update: Self-Efficacy increased to {}",
                virtues.self_efficacy
            );

            // Placeholder: If the history length is even, boost Self-Esteem (simulating a "good" outcome)
            if progress.history.len() % 2 == 0 {
                virtues.self_esteem = (virtues.self_esteem + 0.02).min(1.0);
                info!(
                    "Virtue Update: Self-Esteem increased to {}",
                    virtues.self_esteem
                );
            }

            // Placeholder: Interdependence grows slowly over time/steps
            virtues.interdependence = (virtues.interdependence + 0.01).min(1.0);
        }
    }
}

pub fn monitor_cognitive_load(
    mut query: Query<(&mut CognitiveLoad, &StoryProgress)>,
    time: Res<Time>,
) {
    for (mut load, _progress) in query.iter_mut() {
        // Simulate dynamic cognitive load
        // Intrinsic load could be based on the complexity of the current step (not yet modeled)
        // Extraneous load could decay over time if the UI is "smooth"

        // Simple simulation: Germane load (learning) oscillates slightly
        let oscillation = (time.elapsed_seconds() * 0.5).sin() * 0.05;
        load.germane = (0.5 + oscillation).clamp(0.0, 1.0);

        // Decay extraneous load
        load.extraneous = (load.extraneous - 0.01 * time.delta_seconds()).max(0.0);
    }
}

#[allow(clippy::type_complexity)]
pub fn log_research_events(
    mut query: Query<
        (&mut ResearchLog, &VirtueTopology, &StoryProgress),
        (Changed<VirtueTopology>, Changed<StoryProgress>),
    >,
    time: Res<Time>,
) {
    for (mut log, virtues, progress) in query.iter_mut() {
        // Log Virtue Updates
        // In a real system, we'd diff against previous state, but for now we log snapshots on change
        let event = ResearchEvent {
            timestamp: time.elapsed_seconds_f64(),
            event_type: "VIRTUE_SNAPSHOT".to_string(),
            data: serde_json::to_string(virtues).unwrap_or_default(),
        };
        log.events.push(event);

        // Log Narrative Progress
        if let Some(step) = &progress.current_step_id {
            let event = ResearchEvent {
                timestamp: time.elapsed_seconds_f64(),
                event_type: "NARRATIVE_STEP".to_string(),
                data: format!("Step: {}", step),
            };
            log.events.push(event);
        }

        info!("Research Log Updated: {} events recorded", log.events.len());
    }
}

// System to sync YarnSpinner events to our ECS components
pub fn sync_yarn_to_story_progress(
    mut events: EventReader<DialogueCompleteEvent>,
    mut command_events: EventReader<ExecuteCommandEvent>,
    mut query: Query<&mut VirtueTopology>,
) {
    for _event in events.read() {
        info!("Dialogue Complete!");
    }

    for event in command_events.read() {
        // Parse command: <<add_stat Valor 2>>
        let command = &event.command;
        let parts: Vec<&str> = command.name.split_whitespace().collect();

        if let Some(&"add_stat") = parts.first() {
            if command.parameters.len() >= 2 {
                let stat_name = command.parameters[0].to_string();
                let value_str = command.parameters[1].to_string();

                if let Ok(value) = value_str.parse::<f32>() {
                    for mut virtues in query.iter_mut() {
                        match stat_name.as_str() {
                            "Valor" => virtues.valor += value,
                            "Eloquence" => virtues.competence += value, // Mapping Eloquence to Competence for now
                            "Compassion" => virtues.compassion += value,
                            "SelfEfficacy" => virtues.self_efficacy += value,
                            "Intelligence" => virtues.competence += value, // Mapping Int to Competence
                            "Interdependence" => virtues.interdependence += value,
                            _ => warn!("Unknown stat in Yarn command: {}", stat_name),
                        }
                        info!("Yarn Command Applied: {} +{}", stat_name, value);
                    }
                }
            }
        }
    }
}

// System to sync ECS components to Shared Resources (for Axum)
pub fn sync_ecs_to_shared(
    query: Query<(&ResearchLog, &VirtueTopology), Changed<ResearchLog>>,
    shared_log: Res<crate::game::components::SharedResearchLogResource>,
    shared_virtues: Res<crate::game::components::SharedVirtuesResource>,
) {
    for (log, virtues) in query.iter() {
        if let Ok(mut shared_log_guard) = shared_log.0.write() {
            *shared_log_guard = log.clone();
        }
        if let Ok(mut shared_virtues_guard) = shared_virtues.0.write() {
            *shared_virtues_guard = *virtues;
        }
    }
}
