```toml
# /home/joshua/antigravity/trinity_overnight_work/Cargo.toml
[package]
name = "trinity_avatar"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.6", features = ["csr"] }
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"
log = "0.4"
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Window", "Document"] }

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
```


```rust
// /home/joshua/antigravity/trinity_overnight_work/src/main.rs
use leptos::*;
use trinity_avatar::avatar::{Avatar, AvatarState, Mood};
use trinity_avatar::components::avatar_view::AvatarView;

#[component]
fn App(cx: Scope) -> impl IntoView {
    // Shared avatar state across the UI
    let avatar = create_rw_signal(cx, Avatar::new());

    // Tick every second to update internal timers
    let tick_interval = 1000; // ms
    let _interval = set_interval(
        move || {
            avatar.update(|a| a.tick());
        },
        Duration::from_millis(tick_interval),
    );

    view! { cx,
        <main class="app">
            <h1>"Trinity AI Avatar"</h1>
            <AvatarView avatar=avatar />
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|cx| view! { cx, <App/> })
}
```


```rust
// /home/joshua/antigravity/trinity_overnight_work/src/avatar.rs
use rand::Rng;
use serde::{Deserialize, Serialize};

/// High‑level states the avatar can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvatarState {
    Idle,
    Thinking,
    Coding,
    Learning,
    Resting,
    Energized,
}

/// Mood influences how the avatar behaves and is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    Happy,
    Focused,
    Tired,
    Curious,
    Excited,
}

/// Core data model for an avatar instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Avatar {
    pub current_state: AvatarState,
    pub mood: Mood,
    /// 0‑100 (inclusive). Represents stamina/mental energy.
    pub energy: u8,
    /// Accumulated experience points.
    pub xp: u64,
}

impl Avatar {
    /// Create a fresh avatar with default values.
    pub fn new() -> Self {
        Self {
            current_state: AvatarState::Idle,
            mood: Mood::Curious,
            energy: 100,
            xp: 0,
        }
    }

    /// Attempt to transition to a new state respecting the rules.
    ///
    /// Returns `true` if the transition succeeded, otherwise `false`.
    pub fn transition_to(&mut self, target: AvatarState) -> bool {
        use AvatarState::*;

        let allowed = match (self.current_state, target) {
            // Idle can go anywhere except Energized directly
            (Idle, Thinking)
            | (Idle, Learning)
            | (Idle, Resting) => true,

            // Thinking → Coding or Learning
            (Thinking, Coding) | (Thinking, Learning) => true,

            // Coding → Resting or Energized
            (Coding, Resting) | (Coding, Energized) => true,

            // Learning → Thinking or Resting
            (Learning, Thinking) | (Learning, Resting) => true,

            // Resting → Idle or Energized
            (Resting, Idle) | (Resting, Energized) => true,

            // Energized → Coding or Idle
            (Energized, Coding) | (Energized, Idle) => true,

            _ => false,
        };

        if allowed {
            self.current_state = target;
            // Adjust energy and XP on certain transitions
            match target {
                Thinking => self.energy = self.energy.saturating_sub(5),
                Coding => {
                    self.energy = self.energy.saturating_sub(15);
                    self.xp += 10;
                }
                Learning => {
                    self.energy = self.energy.saturating_sub(8);
                    self.xp += 5;
                }
                Resting => self.energy = (self.energy + 30).min(100),
                Energized => self.energy = 100,
                Idle => {}
            };
            self.update_mood();
        }

        allowed
    }

    /// Re‑evaluate the avatar's mood based on current energy and state.
    pub fn update_mood(&mut self) {
        use Mood::*;
        use AvatarState::*;

        self.mood = match (self.energy, self.current_state) {
            (0..=20, Resting) => Tired,
            (0..=30, _) => Tired,
            (31..=60, Coding | Learning) => Focused,
            (61..=80, Thinking) => Curious,
            (81..=100, Energized) => Excited,
            (_, Idle) => Happy,
            _ => Happy,
        };
    }

    /// Called once per tick (e.g., each second). Handles passive changes.
    pub fn tick(&mut self) {
        // Energy slowly drains when not resting
        if self.current_state != AvatarState::Resting && self.energy > 0 {
            self.energy = self.energy.saturating_sub(1);
        }

        // Random chance to become Energized while idle
        if self.current_state == AvatarState::Idle && self.energy >= 90 {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.02) {
                self.transition_to(AvatarState::Energized);
            }
        }

        // Auto‑recover mood after each tick
        self.update_mood();
    }
}
```


```rust
// /home/joshua/antigravity/trinity_overnight_work/src/components/avatar_view.rs
use leptos::*;
use crate::avatar::{Avatar, AvatarState, Mood};

#[component]
pub fn AvatarView(
    cx: Scope,
    #[prop(into)] avatar: RwSignal<Avatar>,
) -> impl IntoView {
    // Derive CSS class names from state/mood
    let state_class = move || match avatar.get().current_state {
        AvatarState::Idle => "state-idle",
        AvatarState::Thinking => "state-thinking",
        AvatarState::Coding => "state-coding",
        AvatarState::Learning => "state-learning",
        AvatarState::Resting => "state-resting",
        AvatarState::Energized => "state-energized",
    };

    let mood_class = move || match avatar.get().mood {
        Mood::Happy => "mood-happy",
        Mood::Focused => "mood-focused",
        Mood::Tired => "mood-tired",
        Mood::Curious => "mood-curious",
        Mood::Excited => "mood-excited",
    };

    // UI controls for manual transitions (for demo purposes)
    let transition = move |target: AvatarState| {
        avatar.update(|a| { a.transition_to(target); });
    };

    view! { cx,
        <section class="avatar-panel">
            <div class=format!("avatar {} {}", state_class(), mood_class())>
                // Simple visual placeholder – could be an SVG or img
                <div class="face"></div>
            </div>

            <div class="info">
                <p>"State: " {move || format!("{:?}", avatar.get().current_state)}</p>
                <p>"Mood: "  {move || format!("{:?}", avatar.get().mood)}</p>
                <p>"Energy: " {move || avatar.get().energy} "%" </p>
                <p>"XP: " {move || avatar.get().xp}</p>
            </div>

            <div class="controls">
                // Buttons for each possible transition
                <button on:click=move |_| transition(AvatarState::Idle)>"Idle"</button>
                <button on:click=move |_| transition(AvatarState::Thinking)>"Think"</button>
                <button on:click=move |_| transition(AvatarState::Coding)>"Code"</button>
                <button on:click=move |_| transition(AvatarState::Learning)>"Learn"</button>
                <button on:click=move |_| transition(AvatarState::Resting)>"Rest"</button>
                <button on:click=move |_| transition(AvatarState::Energized)>"Energize"</button>
            </div>
        </section>

        // Include CSS (Leptos will inline it when compiled with the `style` feature)
        <link rel="stylesheet" href="/static/avatar.css"/>
    }
}
```


```css
/* /home/joshua/antigravity/trinity_overnight_work/static/avatar.css