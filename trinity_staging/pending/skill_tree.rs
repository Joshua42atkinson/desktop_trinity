**File:** `/home/joshua/antigravity/trinity_overnight_work/Cargo.toml`
```toml
[package]
name = "trinity_skill_tree"
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
    "Element",
    "Node",
    "HtmlCanvasElement",
    "CanvasRenderingContext2d",
    "MouseEvent",
] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
```

---

**File:** `/home/joshua/antigravity/trinity_overnight_work/src/main.rs`
```rust
use leptos::*;
use leptos_router::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use web_sys::{MouseEvent, window};

/// ---------------------------
/// Data structures
/// ---------------------------
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    pub name: &'static str,
    pub description: &'static str,
    pub unlocked: bool,
    pub level: u8, // 0..=5
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain {
    pub name: &'static str,
    pub icon: &'static str,
    pub skills: Vec<Skill>,
    pub unlocked: bool,
}

/// ---------------------------
/// Static skill graph definition
/// ---------------------------
fn build_skill_graph() -> Vec<Domain> {
    vec![
        Domain {
            name: "Rust",
            icon: "🦀",
            unlocked: true,
            skills: vec![
                Skill { name: "Ownership", description: "Master the borrow checker.", unlocked: false, level: 0 },
                Skill { name: "Traits", description: "Define shared behavior.", unlocked: false, level: 0 },
                Skill { name: "Async", description: "Write non‑blocking code.", unlocked: false, level: 0 },
                Skill { name: "Macros", description: "Metaprogramming power.", unlocked: false, level: 0 },
            ],
        },
        Domain {
            name: "AI/ML",
            icon: "🤖",
            unlocked: false,
            skills: vec![
                Skill { name: "Linear Regression", description: "", unlocked: false, level: 0 },
                Skill { name: "Neural Networks", description: "", unlocked: false, level: 0 },
                Skill { name: "Reinforcement Learning", description: "", unlocked: false, level: 0 },
            ],
        },
        Domain {
            name: "Game Dev",
            icon: "🎮",
            unlocked: false,
            skills: vec![
                Skill { name: "ECS", description: "", unlocked: false, level: 0 },
                Skill { name: "Physics", description: "", unlocked: false, level: 0 },
                Skill { name: "Shaders", description: "", unlocked: false, level: 0 },
            ],
        },
        Domain {
            name: "Web Dev",
            icon: "🌐",
            unlocked: false,
            skills: vec![
                Skill { name: "HTML/CSS", description: "", unlocked: false, level: 0 },
                Skill { name: "JavaScript", description: "", unlocked: false, level: 0 },
                Skill { name: "WebAssembly", description: "", unlocked: false, level: 0 },
            ],
        },
        Domain {
            name: "System Design",
            icon: "🖥️",
            unlocked: false,
            skills: vec![
                Skill { name: "Scalability", description: "", unlocked: false, level: 0 },
                Skill { name: "Reliability", description: "", unlocked: false, level: 0 },
                Skill { name: "Observability", description: "", unlocked: false, level: 0 },
            ],
        },
    ]
}

/// ---------------------------
/// Component helpers
/// ---------------------------
fn random_color() -> String {
    let mut rng = rand::thread_rng();
    format!(
        "rgb({},{},{})",
        rng.gen_range(100..256),
        rng.gen_range(100..256),
        rng.gen_range(100..256)
    )
}

/// ---------------------------
/// Main app component
/// ---------------------------
#[component]
fn App(cx: Scope) -> impl IntoView {
    // shared state (graph + view transform)
    let graph = create_rw_signal(cx, build_skill_graph());

    // pan/zoom state
    let offset_x = create_rw_signal(cx, 0.0_f64);
    let offset_y = create_rw_signal(cx, 0.0_f64);
    let scale = create_rw_signal(cx, 1.0_f64);

    // mouse handling for panning
    let is_dragging = create_rw_signal(cx, false);
    let last_mouse = create_rw_signal(cx, (0.0_f64, 0.0_f64));

    let onmousedown = move |ev: MouseEvent| {
        ev.prevent_default();
        is_dragging.set(true);
        last_mouse.set((ev.client_x() as f64, ev.client_y() as f64));
    };

    let onmouseup = move |_ev: MouseEvent| {
        is_dragging.set(false);
    };

    let onmousemove = move |ev: MouseEvent| {
        if is_dragging.get() {
            let (lx, ly) = last_mouse.get();
            let dx = ev.client_x() as f64 - lx;
            let dy = ev.client_y() as f64 - ly;
            offset_x.update(|v| *v += dx);
            offset_y.update(|v| *v += dy);
            last_mouse.set((ev.client_x() as f64, ev.client_y() as f64));
        }
    };

    // zoom with wheel
    let onwheel = move |ev: web_sys::WheelEvent| {
        ev.prevent_default();
        const ZOOM_FACTOR: f64 = 0.001;
        let delta = ev.delta_y() * ZOOM_FACTOR;
        scale.update(|s| {
            let new_s = (*s - delta).max(0.3).min(3.0);
            *s = new_s;
        });
    };

    // unlock a skill (demo animation)
    let unlock_skill = move |domain_idx: usize, skill_idx: usize| {
        graph.update(|g| {
            if let Some(domain) = g.get_mut(domain_idx) {
                if let Some(skill) = domain.skills.get_mut(skill_idx) {
                    if !skill.unlocked {
                        skill.unlocked = true;
                        skill.level = 1;
                    } else if skill.level < 5 {
                        skill.level += 1;
                    }
                }
            }
        });
    };

    view! { cx,
        <div class="app"
            on:mousedown=onmousedown
            on:mouseup=onmouseup
            on:mousemove=onmousemove
            on:wheel=onwheel>
            // background grid
            <svg class="grid" width="100%" height="100%">
                <defs>
                    <pattern id="smallGrid" width="20" height="20" patternUnits="userSpaceOnUse">
                        <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#444" stroke-width="0.5"/>
                    </pattern>
                    <pattern id="grid" width="100" height="100" patternUnits="userSpaceOnUse">
                        <rect width="100" height="100" fill="url(#smallGrid)"/>
                        <path d="M 100 0 L 0 0 0 100" fill="none" stroke="#666" stroke-width="1"/>
                    </pattern>
                </defs>
                <rect width="100%" height="100%" fill="url(#grid)"/>
            </svg>

            // main skill graph
            <svg class="graph"
                 style=move || format!("transform: translate({}px, {}px) scale({});",
                                      offset_x.get(),
                                      offset_y.get(),
                                      scale.get())>
                {move || {
                    let domains = graph.get();
                    let mut elements = vec![];
                    // layout constants
                    let radius = 40.0;
                    let h_spacing = 300.0;
                    let v_spacing = 180.0;

                    for (d_idx, domain) in domains.iter().enumerate() {
                        let x = d_idx as f64 * h_spacing + 100.0;
                        // Domain node
                        elements.push(view! { cx,
                            <g class="domain-node"
                               data-index=d_idx
                               on:click=move |_| {
                                   // unlock domain for demo purposes
                                   graph.update(|g| {
                                       if let Some(dom) = g.get_mut(d_idx) {
                                           dom.unlocked = true;
                                       }
                                   });
                               }>
                                <circle cx=x cy=100 r=radius fill=if domain.unlocked { "goldenrod" } else { "#555" } stroke="#fff" stroke-width="3"/>
                                <text x=x y=108 text-anchor="