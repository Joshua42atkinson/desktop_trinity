use bevy::prelude::*;
use iron_road_physics::{calculate_velocity, CognitiveLoad, Node, Train, VocabularyTier};
use rand::Rng; // For d20 rolls
use serde::{Deserialize, Serialize}; // Imported from Physics Engine

pub struct VaaMPlugin;

impl Plugin for VaaMPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<VocabularyItem>()
            .register_type::<CognitiveWeight>()
            .register_type::<SemanticSocket>()
            .register_type::<MasteryStatus>()
            .add_event::<LogisticsCheckRequest>()
            .add_event::<LogisticsCheckResult>()
            .add_systems(
                Update,
                (
                    logistics_check_system,
                    physics_tick_system,
                    mastery_tracking_system,
                ),
            );
    }
}

// ----------------------------------------------------------------------------
// Components: The Atoms of Language
// ----------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct VocabularyItem {
    pub word: String,
    pub definition: String,
    pub tier: VocabularyTier,
    // Tags for identifying socket compatibility (e.g., "Time", "Emotion")
    pub tags: Vec<String>,
}

#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct CognitiveWeight {
    /// Intrinsic Load (1-100)
    pub base_mass: f32,
    /// Effective Load (Drops to 0 if mastered)
    pub effective_mass: f32,
}

// We implement From for easier conversion if we want to use the pure struct logic
impl From<CognitiveLoad> for CognitiveWeight {
    fn from(load: CognitiveLoad) -> Self {
        Self {
            base_mass: load.base_mass,
            effective_mass: load.effective_mass,
        }
    }
}

#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct SemanticSocket {
    /// Tags required to fit this socket
    pub required_tags: Vec<String>,
    /// Difficulty Class (Bloom's Taxonomy Level)
    pub difficulty_class: u32,
    /// Is this socket currently solved?
    pub is_solved: bool,
}

#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MasteryStatus {
    pub acquisition_count: u32,
    pub application_count: u32,
    pub reinforcement_count: u32,
    pub is_mastered: bool,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TrainComponent(pub Train);

impl Default for TrainComponent {
    fn default() -> Self {
        Self(Train::new(10.0))
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct LessonNodeComponent(pub Node);

impl Default for LessonNodeComponent {
    fn default() -> Self {
        Self(Node::new(10.0, 1.0))
    }
}

// ----------------------------------------------------------------------------
// Events
// ----------------------------------------------------------------------------

#[derive(Event)]
pub struct LogisticsCheckRequest {
    pub entity: Entity,
    pub target_socket: Option<Entity>, // Optional for now, but UI sends it
}

#[derive(Event)]
pub struct LogisticsCheckResult {
    pub entity: Entity,
    pub success: bool,
    pub rolled: f32,
    pub dc: f32,
}

// ----------------------------------------------------------------------------
// Systems
// ----------------------------------------------------------------------------

/// The "Physics" Engine: Calculates if a word fits a socket
/// The "Physics" Engine: Calculates if a word fits a socket
/// Formula: d20 + (Intelligence) - (Friction) >= DC
fn logistics_check_system(
    mut events: EventReader<LogisticsCheckRequest>,
    mut results: EventWriter<LogisticsCheckResult>,
    mut train_query: Query<&mut TrainComponent>,
    node_query: Query<&LessonNodeComponent>,
    socket_query: Query<&SemanticSocket>,
) {
    let mut rng = rand::thread_rng();

    for request in events.read() {
        // 1. Get current Train state (for Coal/Motivation)
        let Ok(mut train_wrapper) = train_query.get_single_mut() else {
            warn!("Logistics Check Failed: No Train component found!");
            continue;
        };
        let train = &mut train_wrapper.0;

        // 2. Get current Node friction
        let friction_penalty = if let Ok(node_wrapper) = node_query.get_single() {
            node_wrapper.0.friction
        } else {
            0.0 // Default to no friction if no node found
        };

        // 3. Roll d20
        let d20: i32 = rng.gen_range(1..=20);

        // 4. Calculate Modifiers
        // Placeholder: Intelligence stat would come from a Persona component
        let intelligence_mod = 0.0;

        // Friction is a penalty
        let modifier = intelligence_mod - friction_penalty;

        let total = d20 as f32 + modifier;

        // 5. Determine Difficulty Class (DC)
        // Ideally this comes from the SemanticSocket component on the target entity
        // For now, we assume a standard DC 10
        let dc = 10.0;

        info!(
            "🎲 Logistics Check: Roll({}) + Mod({:.1}) = {:.1} vs DC {:.1}",
            d20, modifier, total, dc
        );

        if total >= dc {
            info!("✅ SUCCESS! Generating Steam...");
            // Consume Coal (Attention) to generate Steam (Learning)
            if train.stoke_fire(5.0) {
                info!("   Coal consumed. Steam pressure rising.");
            } else {
                warn!("   ⚠️ Not enough Coal (Motivation) to process this!");
            }
            results.send(LogisticsCheckResult {
                entity: request.entity,
                success: true,
                rolled: total,
                dc,
            });
        } else {
            info!("❌ FAILURE! Confusion increases.");
            results.send(LogisticsCheckResult {
                entity: request.entity,
                success: false,
                rolled: total,
                dc,
            });
        }
    }
}

/// Enforces the "Rule of Three" pedagogy
/// 1. Discovery (Acquisition)
/// 2. Application (Logistics Check)
/// 3. Reinforcement (Mastery)
fn mastery_tracking_system(
    mut events: EventReader<LogisticsCheckResult>,
    mut query: Query<(&mut MasteryStatus, &mut CognitiveWeight)>,
) {
    for event in events.read() {
        if event.success {
            if let Ok((mut mastery, mut weight)) = query.get_mut(event.entity) {
                mastery.application_count += 1;
                info!(
                    "📈 Mastery Progress for Entity {:?}: {}/3 Applications",
                    event.entity, mastery.application_count
                );

                // Check for Mastery Threshold (Rule of Three)
                if mastery.application_count >= 3 && !mastery.is_mastered {
                    mastery.is_mastered = true;
                    // Fluent words have 0 effective mass (Automaticity)
                    weight.effective_mass = 0.0;
                    info!(
                        "🎉 MASTERY UNLOCKED! Entity {:?} is now Fluent (0 Mass).",
                        event.entity
                    );
                }
            }
        }
    }
}

/// Updates the physics state of the Train based on the Node it is traversing
fn physics_tick_system(
    mut train_query: Query<&mut TrainComponent>,
    node_query: Query<&LessonNodeComponent>,
    time: Res<Time>,
) {
    if let Ok(mut train_wrapper) = train_query.get_single_mut() {
        // For prototype, assume we are always on the first available node or a default one
        // In reality, this would be based on the Train's current location in the graph
        if let Ok(node_wrapper) = node_query.get_single() {
            let train = &mut train_wrapper.0;
            let node = &node_wrapper.0;

            // Calculate velocity (Client side physics)
            // Note: calculate_velocity mutations are per-tick, but we might want to scale steam consumption by delta time here
            // For now, we just run the raw physics step
            let _velocity = calculate_velocity(train, node);

            // Optional: Decay steam or coal based on time if not handled in core
            // iron-road-physics might need a 'tick(dt)' method later
        }
    }
}
