# Self-Coding Agent

## Source

Research: Section A.4 "Future Outlook"

## Concept
>
> "An agent with access to its own source code and a compiler could potentially modify its own Behavior Trees or prompt strategies."

Trinity modifies its own code using LLM + cargo check verification loop.

## Already Built

- `orchestrator.rs` - Multi-agent dispatch
- `AutonomousRuntime` - Task queue
- Phase 3 in task.md: Self-Coding Boot

## Remaining

1. Workspace scanner (find TODOs/FIXMEs)
2. Generate patches via LLM
3. `cargo check` verification
4. Human approval gate
5. Git commit integration

## Value

**THE DREAM** - Trinity becomes truly autonomous, improving itself.

## Effort

8-16 hours (spread across multiple sessions)

## Priority

HIGH - This is the core vision
