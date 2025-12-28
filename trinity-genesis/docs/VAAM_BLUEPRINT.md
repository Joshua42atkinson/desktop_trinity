# Vocabulary-as-a-Mechanic (VaaM) - Architectural Blueprint

**Executive Summary**: VaaM transfoms language acquisition into a physics-based game mechanic. Words are not strings; they are physical entities with **Mass** (Intrinsic Load) and **Friction** (Extraneous Load) that must be moved via **Combustion** (Germane Load/Steam).

## 1. Core Philosophy: Systems Isomorphism

* **Rust Memory Safety** = **Psychological Safety** (A crash-proof mind).
* **ECS Architecture** = **Situated Cognition** (Words are tools with functional components).
* **Local-First** = **Privacy Moat** (Shadow work stays on device).

## 2. The Physics of Cognitive Load (Iron Road Mechanics)

The system models mental energy as thermodynamics:

* **Mass (Intrinsic Load)**: The inherent difficulty of a word (1-100).
  * *System*: `TrainCar` has `max_capacity`. Too much "Mass" prevents movement.
* **Friction (Extraneous Load)**: Wasted effort from poor design or confusion.
  * *System*: `FrictionPenalty` reduces `LogisticsCheck` rolls.
* **Steam (Germane Load)**: Successful application of schemas.
  * *System*: Solving puzzles converts Coal (Attention) into Steam (XP/Velocity).

## 3. ECS Architecture (Bevy Implementation)

### 3.1 Atoms of Language (Components)

```rust
#[derive(Component)]
pub struct VocabularyItem {
    pub word: String,
    pub definition: String,
    pub tier: VocabularyTier, // Basic, Academic, Hazardous
    pub tags: HashSet<String>, // "Time", "Decay", "Emotion"
}

#[derive(Component)]
pub struct CognitiveWeight {
    pub base_mass: u32,       // 1-100
    pub effective_mass: u32,  // drops to 0 upon mastery
}

#[derive(Component)]
pub struct SemanticSocket {
    pub required_tags: Vec<String>, // e.g. ["Stability"] needed to fix bridge
    pub difficulty_class: u32,      // DC 5-30 (Bloom's Taxonomy)
}
```

### 3.2 The Interaction Loop

1. **DragEndEvent**: Player drags a `VocabularyItem` to a `SemanticSocket`.
2. **Tag Verification**: System checks if `Item.tags` intersects `Socket.required_tags`.
    * *Match*: Trigger `LogisticsCheck`.
    * *Mismatch*: Trigger Feedback ("This word does not fit").
3. **Logistics Check**: `Roll(d20) + Stats - Friction >= DC`.
    * *Success*: Convert Coal -> Steam.
    * *Failure*: Increase Friction (Confusion).

## 4. The Weigh Station (Data Pipeline)

* **Ingestion**: Designer submits word list.
* **Weighing**: LLM (Scout) analyzes words to assign `Mass`, `Tier`, and `Tags`.
* **Storage**: Persisted in `vocabulary_bank` (PostgreSQL/SQLite).

## 5. Interface ("Glass" Sandbox)

* User Interface renders as a "Glass" overlay (EgUI/Pickle) above the 3D world.
* **Inventory**: Limited to $7 \pm 2$ slots (Miller's Law).
