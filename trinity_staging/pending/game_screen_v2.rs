```rust
// /home/joshua/antigravity/trinity_overnight_work/src/main.rs
use leptos::*;
use crate::ui::main_menu::MainMenu;

fn main() {
    // mount the Leptos app
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, <App/> })
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <link rel="stylesheet" href="/static/style.css"/>
        <MainMenu/>
    }
}
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/ui/mod.rs
pub mod main_menu;
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/ui/main_menu.rs
use leptos::*;
use web_sys::HtmlAudioElement;

#[component]
pub fn MainMenu(cx: Scope) -> impl IntoView {
    // preload sound effects
    let hover_sound = create_node_ref::<HtmlAudioElement>(cx);
    let click_sound = create_node_ref::<HtmlAudioElement>(cx);

    // animation state
    let selected = create_rw_signal(cx, 0usize);
    let menu_items = vec![
        ("Continue", "continue"),
        ("New Simulation", "new_sim"),
        ("Knowledge Graph", "graph"),
        ("Quest Log", "quests"),
        ("Settings", "settings"),
        ("Exit", "exit"),
    ];

    // play sound helpers
    let play_hover = move || {
        if let Some(el) = hover_sound.get_unchecked() {
            let _ = el.clone_node_with_deep(true);
            let _ = el.play();
        }
    };
    let play_click = move || {
        if let Some(el) = click_sound.get_unchecked() {
            let _ = el.clone_node_with_deep(true);
            let _ = el.play();
        }
    };

    view! { cx,
        <div class="main-menu">
            // sound elements (hidden)
            <audio node_ref=hover_sound src="/static/sfx/hover.wav" preload="auto"/>
            <audio node_ref=click_sound src="/static/sfx/click.wav" preload="auto"/>

            <h1 class="title">"TRINITY AI OS"</h1>
            <ul class="menu-list">
                {move || menu_items.iter().enumerate().map(|(i, (label, id))| {
                    let is_selected = move || selected.get() == i;
                    view! { cx,
                        <li
                            class=move || if is_selected() { "menu-item selected" } else { "menu-item" }
                            on:mouseenter=move |_| {
                                selected.set(i);
                                play_hover();
                            }
                            on:click=move |_| {
                                play_click();
                                // placeholder navigation logic
                                log::info!("Menu item clicked: {}", id);
                            }
                        >
                            {label}
                        </li>
                    }
                }).collect_view()}
            </ul>

            <div class="footer">
                "© 2025 Trinity Labs – All rights reserved."
            </div>
        </div>
    }
}
```

```css
/* /home/joshua/antigravity/trinity_overnight_work/static/style.css */
@import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@400;700&display=swap');

:root {
  --bg-dark: #0a0a0c;
  --bg-panel: rgba(15, 15, 20, 0.85);
  --accent-primary: #00e6ff;
  --accent-secondary: #7affd1;
  --text-primary: #e0e0e0;
  --text-muted: #888;
  --glow-radius: 8px;
}

/* Global resets */
*,
*::before,
*::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}
html, body {
  height: 100%;
  font-family: 'Orbitron', sans-serif;
  background: var(--bg-dark);
  color: var(--text-primary);
  overflow: hidden;
}

/* Main menu container */
.main-menu {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2rem;
  background: radial-gradient(circle at top left, rgba(0,230,255,.12), transparent);
}

/* Title */
.title {
  font-size: 3.5rem;
  letter-spacing: .15em;
  text-shadow:
    0 0 var(--glow-radius) var(--accent-primary),
    0 0 calc(var(--glow-radius)*2) var(--accent-secondary);
  animation: pulse 4s infinite ease-in-out;
}

/* Menu list */
.menu-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.menu-item {
  font-size: 1.8rem;
  padding: .6rem 2rem;
  border-radius: .5rem;
  cursor: pointer;
  position: relative;
  transition: transform .15s, background-color .3s;
  color: var(--text-muted);
}
.menu-item::before {
  content: "";
  position: absolute;
  inset: -4px;
  border-radius: .6rem;
  background: linear-gradient(45deg,
    transparent,
    var(--accent-primary) 30%,
    transparent 60%);
  opacity: 0;
  transition: opacity .3s, transform .3s;
}
.menu-item:hover {
  color: var(--text-primary);
  transform: translateX(.4rem);
}
.menu-item.selected {
  color: var(--text-primary);
  background: var(--bg-panel);
  box-shadow:
    0 0 var(--glow-radius) var(--accent-primary),
    inset 0 0 .5rem var(--accent-secondary);
}
.menu-item.selected::before {
  opacity: .7;
  animation: scan 2s infinite linear;
}

/* Footer */
.footer {
  font-size: .85rem;
  color: var(--text-muted);
  margin-top: 4rem;
}

/* Animations */
@keyframes pulse {
  0%,100% { text-shadow:
    0 0 var(--glow-radius) var(--accent-primary),
    0 0 calc(var(--glow-radius)*2) var(--accent-secondary); }
  50% { text-shadow:
    0 0 calc(var(--glow-radius)*1.5) var(--accent-primary),
    0 0 calc(var(--glow-radius)*3) var(--accent-secondary); }
}
@keyframes scan {
  0%   { transform: translateX(-100%); }
  100% { transform: translateX(200%); }
}

/* Responsive */
@media (max-width: 768px) {
  .title { font-size: 2.8rem; }
  .menu-item { font-size: 1.4rem; }
}
```

```toml
# /home/joshua/antigravity/trinity_overnight_work/Cargo.toml
[package]
name = "trinity_overnight_work"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.6", features = ["csr"] }
console_error_panic_hook = "0.1"
log = "0.4"
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["HtmlAudioElement"] }

[lib]
crate-type = ["cdylib"]
```

```html
<!-- /home/joshua/antigravity/trinity_overnight_work/static/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <title>Trinity AI OS</title>
    <script type="module">
        import init from './pkg/trinity_overnight_work.js';
        init();
    </script>
</head>
<body></body>
</html>
```