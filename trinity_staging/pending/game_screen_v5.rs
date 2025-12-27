```rust
// src/main.rs
use leptos::*;
use crate::app::App;

fn main() {
    // mount the Leptos app
    mount_to_body(|| view! { <App/> })
}
```

```rust
// src/app.rs
use leptos::*;
mod components;
use components::main_menu::MainMenu;

/// Root component – switches between screens (only MainMenu for now)
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // future state handling could be added here
    view! { cx,
        <link rel="stylesheet" href="/static/style.css"/>
        <MainMenu/>
    }
}
```

```rust
// src/components/mod.rs
pub mod main_menu;
```

```rust
// src/components/main_menu.rs
use leptos::*;
use web_sys::HtmlAudioElement;

/// Helper to play a sound effect (fire‑and‑forget)
fn play_sfx(src: &str) {
    let window = web_sys::window().unwrap();
    if let Ok(audio) = HtmlAudioElement::new_with_src(src) {
        // clone for the closure
        let audio_clone = audio.clone();
        let _ = audio.play(); // ignore promise errors
        // automatically pause after it ends (prevents memory leak)
        let onended = Closure::wrap(Box::new(move || {
            let _ = audio_clone.pause();
        }) as Box<dyn FnMut()>);
        audio.set_onended(Some(onended.as_ref().unchecked_ref()));
        onended.forget();
    }
}

#[component]
pub fn MainMenu(cx: Scope) -> impl IntoView {
    // sound effect URLs (place them in /static/sfx/)
    const SELECT_SFX: &str = "/static/sfx/select.wav";
    const HOVER_SFX: &str = "/static/sfx/hover.wav";

    let start_game = move |_| {
        play_sfx(SELECT_SFX);
        // TODO: transition to game scene
        log!("Start Game pressed");
    };
    let load_game = move |_| {
        play_sfx(SELECT_SFX);
        log!("Load Game pressed");
    };
    let settings = move |_| {
        play_sfx(SELECT_SFX);
        log!("Settings pressed");
    };
    let exit = move |_| {
        play_sfx(SELECT_SFX);
        log!("Exit pressed");
    };

    // hover sound hook
    let on_hover = move |_ev: ev::MouseEvent| {
        play_sfx(HOVER_SFX);
    };

    view! { cx,
        <div class="main-menu">
            <div class="title">"TRINITY AI OS"</div>
            <ul class="menu-list">
                <li on:click=start_game on:mouseenter=on_hover class="menu-item">"Start Game"</li>
                <li on:click=load_game on:mouseenter=on_hover class="menu-item">"Load Game"</li>
                <li on:click=settings on:mouseenter=on_hover class="menu-item">"Settings"</li>
                <li on:click=exit on:mouseenter=on_hover class="menu-item">"Exit"</li>
            </ul>
        </div>
    }
}
```

```css
/* static/style.css */

/* ---- Global dark sci‑fi palette ---- */
:root {
    --bg-primary: #0a0a12;
    --bg-panel:   rgba(15, 15, 25, 0.85);
    --accent:     #00ffea;
    --text-main:  #e0e0f8;
    --text-muted: #7a7ab2;
}

/* ---- Base reset ---- */
html, body {
    margin:0; padding:0;
    width:100%; height:100%;
    background:#000;
    font-family:"Segoe UI",Helvetica,Arial,sans-serif;
    color:var(--text-main);
    overflow:hidden;
}
a { text-decoration:none; color:inherit; }

/* ---- Main Menu Layout ---- */
.main-menu {
    position:absolute;
    inset:0;
    display:flex;
    flex-direction:column;
    align-items:center;
    justify-content:center;
    background:linear-gradient(135deg, #000428 0%, #004e92 100%);
}

/* Title with subtle neon glow */
.title {
    font-size:3.2rem;
    letter-spacing:0.15em;
    margin-bottom:4rem;
    color:var(--accent);
    text-shadow:
        0 0 8px var(--accent),
        0 0 16px var(--accent),
        0 0 24px var(--accent);
}

/* Menu list */
.menu-list {
    list-style:none;
    padding:0;
    margin:0;
    display:grid;
    gap:1.2rem;
}

/* Individual menu items */
.menu-item {
    font-size:1.8rem;
    padding:.6rem 2rem;
    background:var(--bg-panel);
    border-radius:8px;
    cursor:pointer;
    text-align:center;
    transition:
        transform .15s ease,
        box-shadow .15s ease,
        background-color .2s;
}

/* Hover / focus animation */
.menu-item:hover,
.menu-item:focus {
    background:rgba(0,255,234,.12);
    color:#fff;
    transform:scale(1.07);
    box-shadow:
        0 0 12px var(--accent),
        inset 0 0 8px var(--accent);
}

/* Pressed state */
.menu-item:active {
    transform:scale(.98);
}

/* Fade‑in animation on load */
@keyframes fadeInUp {
    from { opacity:0; transform:translateY(30px); }
    to   { opacity:1; transform:none; }
}
.main-menu .menu-item {
    animation:fadeInUp .6s ease forwards;
}
.main-menu .menu-item:nth-child(1){ animation-delay:.2s; }
.main-menu .menu-item:nth-child(2){ animation-delay:.4s; }
.main-menu .menu-item:nth-child(3){ animation-delay:.6s; }
.main-menu .menu-item:nth-child(4){ animation-delay:.8s; }

/* ---- Responsive tweaks ---- */
@media (max-width:768px) {
    .title { font-size:2.5rem; }
    .menu-item { font-size:1.5rem; padding:.5rem 1.5rem; }
}
```

```toml
# Cargo.toml (relevant sections)
[package]
name = "trinity_overnight_work"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.6", features = ["csr"] }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["HtmlAudioElement", "Window"] }

[lib]
crate-type = ["cdylib", "rlib"]
```

```html
<!-- static/index.html (served as entry point) -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <title>Trinity AI OS</title>
    <script type="module">
        import init from '/pkg/trinity_overnight_work.js';
        init();
    </script>
</head>
<body></body>
</html>
```

```bash
# Build script (run in project root)
wasm-pack build --target web --out-dir ./static/pkg
```

Place the sound files `select.wav` and `hover.wav` inside `static/sfx/`. The UI will animate on load, provide hover/selection sound hooks, and use a dark sci‑fi theme reminiscent of AAA game menus.