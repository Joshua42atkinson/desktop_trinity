# Ask Pete (The Educator) - ADDIE Analysis

## Analysis (Maturity Assessment)

**Status**: **[Maturity Level: 1/5 - Skeleton Only]**
"Ask Pete" is the branding for the pedagogical capability of Trinity, implemented technically as the `Educator` skill (`crates/trinity-skills/src/educator.rs`). Currently, it is a very thin wrapper around an LLM prompt.

### Strengths

* **Structured Output**: It uses `trinity-kernel::GrammarSpec::Json` to force the LLM to return valid JSON for Quizzes and Labs. This is a critical foundation for automated grading.
* **Persona**: The system prompt (`"You are an expert Professor..."`) is established in the code.

### Friction Points

* **No State**: The `Educator` skill is stateless. It generates a Quiz, but it doesn't know *who* the student is, what they struggled with yesterday, or if they passed.
* **No Curriculum Graph**: There is no "Knowledge Graph" that maps dependencies (e.g., "Must know Variables before Loops"). It's just a one-off generator.
* **Hardcoding**: The prompts are hardcoded in Rust. A teacher cannot easily change the "Voice" of the Professor without recompiling.

## Design

* **Pattern**: "Constructivist Scaffolding". The AI should generate challenges that are just above the student's current ability (Zone of Proximal Development).
* **Integration**:
  * **Input**: `AssessmentRequest` (Topic, Difficulty, Audience).
  * **Output**: `AssessmentResponse` (JSON Quiz/Lab).

## Development

* **Language**: Rust (Native).
* **Location**: `crates/trinity-skills/src/educator.rs`.

## Implementation

* **Current Capabilities**:
  * Can generate a 5-question multiple choice quiz on any topic.
  * Can generate a "Lab Project" structure (Title, Steps, Starter Code).
* **Missing**:
  * **Grading**: No logic to evaluate student answers.
  * **Adaptive Learning**: No feedback loop to adjust difficulty based on performance.

## Evaluation

* **Next Steps**:
    1. **Statefulness**: Connect `Educator` to `trinity-brain`'s Memory system to recall student history ("You failed the last quiz on loops, let's try an easier one").
    2. **Curriculum Definition**: Create a file format (TOML/YAML) for defining a "Course" (Module 1 -> Module 2) so the Agent knows the path.
    3. **Frontend**: Create a UI in `trinity-client` to actually render the JSON Quiz.
