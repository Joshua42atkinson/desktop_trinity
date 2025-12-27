# Tickless Idle Mode

## Source

Research: "Autonomous Media Synthesis Agents" Section 8.2

## Concept

- **Active State**: 60 Hz tick rate (rendering/encoding)
- **Idle State**: 1 Hz tick rate (monitoring)

When the Behavior Tree enters Idle, reduce loop frequency to save 98% CPU.

## Implementation

```rust
// In ScheduleRunnerPlugin config
fn set_idle_mode(rate: f64) {
    // rate = 60.0 for active, 1.0 for idle
}
```

## Value

- 24/7 operation without burning electricity
- Lower heat = longer hardware life
- Still responsive to wake events

## Effort

1-2 hours

## Priority

Medium - implement after basic always-on works
