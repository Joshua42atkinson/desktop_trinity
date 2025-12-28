# Trinity Tools (Skills) - ADDIE Analysis

## Analysis (Maturity Assessment)

**Status**: **[Maturity Level: 3/5 - Functional Framework]**
The "Skills" system (mapped to `crates/trinity-skills`) acts as the "Hands" of the system. It is a library of specialized capabilities (Coder, Writer, Educator) that the Brain can invoke.

### Strengths

* **Modular Design**: Each skill (`Coder`, `Writer`, `Educator`) is its own struct with a clear `new()` constructor and `generate()` method.
* **Type Safety**: Heavily uses `trinity-protocol` types to ensure the Brain and Skills speak the same language.
* **Grammar Enforcement**: Uniquely integrates `trinity-kernel::GrammarSpec` to force the LLM to output valid JSON or Rust code, which is a significant robustness feature.

### Friction Points

* **WASM Ambiguity**: There is a `trinity-skills` crate (Rust native) AND a `quadradical-tools` directory (WASM plugins). It is unclear which is the "primary" tool path. The Brain loads WASM plugins *and* uses native skills.
* **Testing**: Skills like `Coder` have basic syntax checks, but no deep semantic verification (e.g., does the code compile?) within the skill itself—it relies on the `ToolExecutor` in the kernel for that.
* **Hardcoded Prompts**: System prompts are hardcoded as `const` strings in the Rust files (e.g., `DEFAULT_CODER_SYSTEM_PROMPT`), making them hard to iterate on without recompiling.

## Design

* **Pattern**: "Command Pattern". The Brain constructs a Request object, passes it to a Skill, and gets a Response object.
* **Philosophy**: "Do One Thing Well". The `Coder` skill doesn't know about Biology; the `Educator` skill doesn't know about `cargo build`.

## Development

* **Language**: Rust (Native).
* **Modules**:
  * `coder.rs`: Generates code.
  * `writer.rs`: Generates markdown content.
  * `media/`: Generates images (SDXL via Candle).
  * `web.rs`: (Planned) Web browsing.

## Implementation

* **Execution**: Skills are library calls made by `trinity-brain`'s RPC handlers. `BrainService::generate_code` instantiates a `Coder`, calls `generate`, and returns the result.
* **State**: Skills are mostly stateless; they rely on the `Brain` trait for intelligence.

## Evaluation

* **Next Steps**:
    1. **Unify Tools**: Clarify the relationship between Native Skills (`trinity-skills`) and WASM Tools (`plugins/`). Should `Coder` become a WASM plugin?
    2. **Externalize Prompts**: Move system prompts to a config file (`prompts.toml`) so they can be tweaked by users/teachers.
    3. **Sandboxing**: Ensure `trinity-skills` (specifically file I/O in `tools`) respects the same safety boundaries as the WASM sandbox.
