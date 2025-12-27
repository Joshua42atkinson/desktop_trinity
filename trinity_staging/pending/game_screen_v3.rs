```rust
// src/main.rs
use leptos::*;
use leptos_router::*;

mod components;
mod assets;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx,
        <Router>
            <Route path="" view=|| view!{ cx, <components::MainMenu/> }/>
        </Router>
    });
}
```

```rust
// src/components/mod.rs
pub mod main_menu;
pub use main_menu::MainMenu;
```

```rust
// src/components/main_menu.rs
use leptos::*;
use crate::assets::SOUND_CLICK;

#[component]
pub fn MainMenu(cx: Scope) -> impl IntoView {
    // Helper to play a click sound (hooked into UI actions)
    let play_click = move || {
        let audio = web_sys::HtmlAudioElement::new_with_src(SOUND_CLICK).unwrap();
        let _ = audio.play();
    };

    view! { cx,
        <div class="ui-screen">
            <div class="bg-grid"></div>
            <section class="logo">
                "TRINITY AI OS"
            </section>

            <nav class="menu">
                <button on:click=move |_| {
                    play_click();
                    // TODO: navigate to game screen
                }>"Start Simulation"</button>
                <button on:click=move |_| {
                    play_click();
                    // TODO: open settings modal
                }>"Settings"</button>
                <button on:click=move |_| {
                    play_click();
                    // TODO: open quest log
                }>"Quest Log"</button>
                <button on:click=move |_| {
                    play_click();
                    // TODO: exit
                }>"Exit"</button>
            </nav>

            <footer class="credits">
                "© 2025 Trinity Labs – All Rights Reserved"
            </footer>
        </div>
    }
}
```

```rust
// src/assets/mod.rs
pub const SOUND_CLICK: &str = "/assets/sfx/click.mp3";
```

```css
/* static/style.css */
@import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@400;700&display=swap');

:root {
    --primary:#0ff;
    --accent:#ff00ff;
    --bg-dark:#050505;
    --bg-grid:#111;
    --text:#e0e0e0;
    --glow:rgba(0,255,255,.2);
}

/* Global resets */
html, body, #root {
    height:100%;
    margin:0;
    padding:0;
    background:var(--bg-dark);
    color:var(--text);
    font-family:'Orbitron',sans-serif;
    overflow:hidden;
}

/* Grid overlay animation */
.bg-grid {
    position:absolute;
    inset:0;
    background-image:
        linear-gradient(0deg, var(--bg-grid) 1px, transparent 1px),
        linear-gradient(90deg, var(--bg-grid) 1px, transparent 1px);
    background-size:40px 40px;
    animation: drift 30s linear infinite;
    pointer-events:none;
}
@keyframes drift {
    from {background-position:0 0;}
    to   {background-position:40px 40px;}
}

/* UI container */
.ui-screen {
    position:relative;
    width:100%;
    height:100%;
    display:flex;
    flex-direction:column;
    align-items:center;
    justify-content:center;
    gap:2rem;
    text-align:center;
}

/* Logo */
.logo {
    font-size:3rem;
    color:var(--primary);
    text-shadow:
        0 0 10px var(--primary),
        0 0 20px var(--primary),
        0 0 30px var(--accent);
    animation: pulse 4s ease-in-out infinite;
}
@keyframes pulse {
    0%,100% { opacity:1; }
    50%      { opacity:.6; }
}

/* Menu */
.menu {
    display:flex;
    flex-direction:column;
    gap:1rem;
}
.menu button {
    background:none;
    border:2px solid var(--accent);
    color:var(--text);
    padding:.8rem 2rem;
    font-size:1.2rem;
    cursor:pointer;
    position:relative;
    overflow:hidden;
    transition:color .3s, box-shadow .3s;
}
.menu button::before {
    content:"";
    position:absolute;
    inset:0;
    background:linear-gradient(45deg,var(--primary),var(--accent));
    opacity:.2;
    transform:scaleX(0);
    transform-origin:left;
    transition:transform .4s ease-out;
}
.menu button:hover::before { transform:scaleX(1); }
.menu button:hover {
    color:#fff;
    box-shadow:0 0 15px var(--accent);
}

/* Footer */
.credits {
    position:absolute;
    bottom:1rem;
    font-size:.8rem;
    opacity:.6;
}

/* Fade‑in animation for the whole screen */
.ui-screen { animation: fadeIn .8s ease forwards; }
@keyframes fadeIn { from{opacity:0;} to{opacity:1;} }
```

```toml
# Cargo.toml (excerpt)
[package]
name = "trinity_ui"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.6", features = ["csr"] }
leptos_router = "0.6"
console_error_panic_hook = "0.1"
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["HtmlAudioElement"] }
```

```html
<!-- index.html (placed in /static) -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <title>Trinity AI OS</title>
    <link rel="stylesheet" href="/style.css"/>
    <script type="module">
        import init from '/pkg/trinity_ui.js';
        init();
    </script>
</head>
<body>
    <div id="root"></div>
</body>
</html>
```

```bash
# Build script (run in project root)
wasm-pack build --target web --out-dir static/pkg
```

*Place the sound file `click.mp3` under `static/assets/sfx/`.*
