# Trinity Educational AI OS — Usability Audit & TODO

**Date**: 2025-12-25  
**Perspective**: Educator/Stakeholder System Review

---

## Executive Summary

Trinity has a **solid architectural foundation** but lacks the **end-to-end polish** needed for educational deployment. The kernel infrastructure is mature, but skills are stubs, the UI doesn't fully surface capabilities, and critical educator workflows are missing.

```
Current Readiness: ████████░░ 80% Architecture | ██░░░░░░░░ 20% Production
```

---

## Part 1: Stakeholder Personas Analysis

### 👩‍🏫 Educator (Primary User)

**Needs:**

- Submit lesson plans for Trinity to elaborate
- Generate quizzes/assessments with structured output
- Monitor student agent interactions
- Preview educational content artifacts

**Current Gaps:**

- ❌ No structured assessment output (grading artifacts)
- ❌ No curriculum/lesson plan templates
- ❌ No student session visibility
- ❌ Workspace modes exist but aren't connected to educator workflows

### 🧑‍🎓 Student (End User)

**Needs:**

- Clear feedback on questions
- Step-by-step explanations (Planning mode artifacts)
- Voice interaction for accessibility
- Visual progress tracking

**Current Gaps:**

- ❌ Artifacts can't be expanded/collapsed in UI
- ⚠️ TTS exists but not wired to frontend
- ⚠️ Avatar exists but state not driven by real events
- ❌ No "explain like I'm X" mode selector

### 🔧 Developer/Maintainer

**Needs:**

- Clear module boundaries
- Comprehensive tests
- Hot-reload for UI iteration

**Current Gaps:**

- ⚠️ Many skills are placeholder stubs
- ⚠️ Tests exist but some are failing (pre-existing)
- ❌ No integration tests for full workflows

---

## Part 2: Module-by-Module Gaps

### Kernel (trinity-kernel) — 80% Complete

| Module | Status | Gap |
|--------|--------|-----|
| `brain.rs`, `brain_desktop.rs` | ✅ | Working with Vulkan |
| `orchestrator.rs` | ✅ | Multi-agent dispatch works |
| `agent_graph.rs` | ✅ | DAG workflows defined |
| `wasm_sandbox.rs` | ⚠️ | Stubs, no actual wasmtime execution |
| `agent_compiler.rs` | ⚠️ | Stubs, no real WASM generation |
| `npu_backend.rs` | ⚠️ | Skeleton only |
| `rpc_pool.rs` | ⚠️ | Skeleton only |
| `tts.rs` | ⚠️ | Interface only, no zonos integration |
| `voice.rs` | ⚠️ | Types defined, no synthesis |

### Skills (trinity-skills) — 30% Complete

| Module | Status | Gap |
|--------|--------|-----|
| `coder.rs` | ⚠️ | Placeholder—no Brain RPC call |
| `writer.rs` | ⚠️ | Placeholder—no Brain RPC call |
| `web.rs` | ⚠️ | Browse works, search is stub |
| `media/` | ⚠️ | ImageGenerator skeleton |
| `tools/` | ⚠️ | ToolExecutor skeleton |
| `code_editor.rs` | ⚠️ | Placeholder |
| `drive.rs` | ⚠️ | Placeholder (commented out) |

### Body (trinity-body) — 60% Complete

| Module | Status | Gap |
|--------|--------|-----|
| `main.rs` | ✅ | App runs, UI renders |
| `avatar.rs` | ✅ | 5 personas, animations work |
| `bridge.rs` | ⚠️ | 8 request types, missing artifact streaming |
| `panels/antigravity.rs` | ⚠️ | Events display, artifacts not rendered richly |
| `panels/workspace.rs` | ⚠️ | 7 modes defined, not applied to layout |
| `panels/tasks.rs` | ⚠️ | Local list, no Brain sync |
| `audio.rs` | ⚠️ | Resource exists, not connected |

### Protocol (trinity-protocol) — 90% Complete

| Module | Status | Gap |
|--------|--------|-----|
| `artifact.rs` | ✅ | New—AgentMode + Artifact enum |
| `stream.rs` | ✅ | Extended with artifact events |
| `types.rs` | ✅ | Voice, Image, Hardware types |
| `brain.rs` | ✅ | RPC service definition |
| `task.rs` | ✅ | Task service definition |

---

## Part 3: Critical Path for Education

### Phase A: Complete the Skills-to-UI Pipeline (P0)

1. **Wire Coder skill to Brain RPC** — Submit prompts, receive code
2. **Wire Writer skill to Brain RPC** — Submit prompts, receive documents
3. **Emit ArtifactGenerated events** from Orchestrator
4. **Render artifacts richly** in Antigravity panel

### Phase B: Educator Workflows (P1)

5. **Assessment Generator** — Input topic → Output structured quiz (Extractor pattern)
2. **Lesson Plan Assistant** — Input objectives → Output lesson artifact with steps
3. **Grading Rubric** — Structured JSON output for LMS integration

### Phase C: Student Experience (P1)

8. **Voice Integration** — Wire TTS to VoiceResponse in bridge
2. **Avatar State Sync** — Drive avatar from StreamEvent states
3. **"Explain Like I'm X"** — Mode selector for explanation depth

### Phase D: Polish & Reliability (P2)

11. **Workspace Mode Layouts** — Actually apply PanelVisibility to UI
2. **Task Sync with Brain** — Implement ListPendingTasks
3. **WASM Sandbox Production** — Real wasmtime execution
4. **NPU Routing** — Hardware detection and routing

---

## Part 4: Recommended Priority Order

| # | Task | Impact | Effort | Why |
|---|------|--------|--------|-----|
| 1 | Wire skills to Brain RPC | 🔥🔥🔥 | Low | Skills are dead without this |
| 2 | Emit artifacts from Orchestrator | 🔥🔥🔥 | Med | Enables rich UI |
| 3 | Render artifacts in UI | 🔥🔥🔥 | Med | User sees structured output |
| 4 | Assessment Generator | 🔥🔥🔥 | Med | Core educator value prop |
| 5 | Voice/TTS wiring | 🔥🔥 | Med | Accessibility |
| 6 | Avatar state sync | 🔥🔥 | Low | User feedback |
| 7 | Workspace mode layouts | 🔥 | Low | Polish |
| 8 | Task list sync | 🔥 | Med | Queue visibility |
| 9 | WASM production | 🔥 | High | Sandboxed code execution |
| 10 | NPU routing | 🔥 | High | Performance optimization |

---

## Part 5: Architectural Decisions

### Why Rust-Only?

- **No Python runtime on student devices** — Simpler deployment
- **Deterministic memory** — No GC pauses during interaction
- **Single binary** — IT admins can distribute easily
- **WASM security** — Sandboxed student code execution

### Why Bevy?

- **ECS architecture** — Natural fit for agent/UI state
- **3D avatar** — Visual engagement for students
- **Plugin ecosystem** — Extensible for future features

### Why Local Inference?

- **Privacy** — Student data never leaves device
- **Latency** — Instant feedback for learning
- **Cost** — No per-token API fees at scale

---

## Next Steps

1. Choose one item from Phase A
2. Implement end-to-end
3. Verify with educator persona test
4. Iterate

> *"The goal is not to ship features, but to enable learning."*
