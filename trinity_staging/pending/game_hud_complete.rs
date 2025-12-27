**/home/joshua/antigravity/trinity_overnight_work/Cargo.toml**
```toml
[package]
name = "trinity_hud"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.6", features = ["csr"] }
leptos_router = "0.6"
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "Window",
    "Document",
    "HtmlElement",
] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["wasmbind"] }
```

**/home/joshua/antigravity/trinity_overnight_work/src/main.rs**
```rust
use leptos::*;
use crate::components::*;

#[component]
fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="hud-root">
            <TopBar />
            <div class="main-grid">
                <LeftPanel />
                <CenterArea />
                <RightPanel />
            </div>
            <BottomBar />
            <MiniMap />
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, <App/> })
}
```

**/home/joshua/antigravity/trinity_overnight_work/src/components/mod.rs**
```rust
pub mod top_bar;
pub mod left_panel;
pub mod center_area;
pub mod right_panel;
pub mod bottom_bar;
pub mod mini_map;

pub use top_bar::TopBar;
pub use left_panel::LeftPanel;
pub use center_area::CenterArea;
pub use right_panel::RightPanel;
pub use bottom_bar::BottomBar;
pub use mini_map::MiniMap;
```

**/home/joshua/antigravity/trinity_overnight_work/src/components/top_bar.rs**
```rust
use leptos::*;
use chrono::{Local, Timelike};

#[component]
pub fn TopBar(cx: Scope) -> impl IntoView {
    let (time, set_time) = create_signal(cx, Local::now().format("%H:%M").to_string());

    // update every second
    spawn_local({
        let set_time = set_time.clone();
        async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(1000).await;
                let now = Local::now();
                set_time.set(now.format("%H:%M").to_string());
            }
        }
    });

    view! { cx,
        <header class="top-bar">
            <div class="logo">"🜂 Trinity AI"</div>
            <div class="time">{move || time.get()}</div>
            <div class="status-indicators">
                <span class="indicator online">"●"</span>
                <span class="indicator cpu">"CPU 23%"</span>
                <span class="indicator mem">"RAM 58%"</span>
            </div>
        </header>
    }
}
```

**/home/joshua/antigravity/trinity_overnight_work/src/components/left_panel.rs**
```rust
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Avatar {
    mood: String,
    level: u32,
    xp_current: u32,
    xp_next: u32,
}

#[component]
pub fn LeftPanel(cx: Scope) -> impl IntoView {
    // Mock avatar data
    let avatar = Avatar {
        mood: "🤖".into(),
        level: 7,
        xp_current: 3400,
        xp_next: 5000,
    };
    let progress = (avatar.xp_current as f64 / avatar.xp_next as f64) * 100.0;

    view! { cx,
        <aside class="left-panel">
            <div class="avatar">
                <img src="/assets/avatar.png" alt="Avatar"/>
                <div class="mood">{avatar.mood}</div>
            </div>
            <div class="level">{"Level "}{avatar.level}</div>
            <div class="xp-bar">
                <div class="xp-fill" style=format!("width: {}%;", progress)></div>
                <span class="xp-text">{format!("{}/{}", avatar.xp_current, avatar.xp_next)}</span>
            </div>
        </aside>
    }
}
```

**/home/joshua/antigravity/trinity_overnight_work/src/components/center_area.rs**
```rust
use leptos::*;

#[component]
pub fn CenterArea(cx: Scope) -> impl IntoView {
    view! { cx,
        <section class="center-area">
            // Example glassmorphism cards
            <div class="card glass">
                <h3>"System Overview"</h3>
                <p>"All systems nominal. Neural nets operating at 99% efficiency."</p>
            </div>
            <div class="card glass">
                <h3>"Recent Logs"</h3>
                <ul>
                    <li>"[12:03] User login – successful"</li>
                    <li>"[12:07] Data sync completed"</li>
                    <li>"[12:15] Alert: High CPU usage (78%)"</li>
                </ul>
            </div>
        </section>
    }
}
```

**/home/joshua/antigravity/trinity_overnight_work/src/components/right_panel.rs**
```rust
use leptos::*;

#[component]
pub fn RightPanel(cx: Scope) -> impl IntoView {
    view! { cx,
        <aside class="right-panel">
            <section class="task-queue">
                <h4>"Task Queue"</h4>
                <ul>
                    <li>"🟢 Analyze user intent"</li>
                    <li>"🔵 Optimize memory cache"</li>
                    <li>"⚪️ Idle"</li>
                </ul>
            </section>

            <section class="memory-stats">
                <h4>"Memory"</h4>
                <div class="mem-bar">
                    <div class="mem-fill" style="width: 58%;"></div>
                </div>
                <span>"3.2 GB / 5.6 GB"</span>
            </section>

            <section class="quick-actions">
                <button class="qa-btn">"🔄 Refresh"</button>
                <button class="qa-btn">"⚙️ Settings"</button>
                <button class="qa-btn">"❓ Help"</button>
            </section>
        </aside>
    }
}
```

**/home/joshua/antigravity/trinity_overnight_work/src/components/bottom_bar.rs**
```rust
use leptos::*;

#[component]
pub fn BottomBar(cx: Scope) -> impl IntoView {
    view! { cx,
        <footer class="bottom-bar">
            <div class="now-playing">
                "Now Playing: " <span class="track">"Neon Dreams – Synthwave"</span>
            </div>
            <div class="notifications">
                <span class="notif-icon">"🔔"</span>
                <span class="notif-count">2</span>
            </div>
            <div class="quick-settings">
                <button class="qs-btn">"🌓 Dark/Light"</button>
                <button class="qs-btn">"⚡️ Performance"</button>
            </div>
        </footer>
    }
}
```

**/home/joshua/antigravity/trinity_overnight_work/src/components/mini_map.rs**
```rust
use leptos::*;

#[component]
pub fn MiniMap(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="mini-map">
            // Placeholder SVG – replace with dynamic knowledge graph later
            <svg viewBox="0 0 200 200" preserveAspectRatio="xMidYMid meet">
                <circle cx="100" cy="100" r="90" stroke="#00ffea" fill="none" stroke-width="2"/>
                <line x1="100" y1="10" x2="100" y2="190" stroke="#00ffea" stroke-width="1"/>
                <line x1="10" y1="100" x2="190" y2="100" stroke="#00ffea" stroke-width="1"/>
                <text x="105" y="30" fill="#00ffea" font-size="8">"Core"</text>
                <text x="150" y="115" fill="#00ffea" font-size="6">"Module A"</text>
                <text x="50" y="115" fill="#00ffea" font-size="6">"Module B"</text>
            </svg>
        </div>
    }
}
```

**/home/joshua/antigravity/trinity_overnight_work/static/style.css**
```