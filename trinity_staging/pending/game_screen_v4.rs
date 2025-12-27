**`Cargo.toml`**
```toml
[package]
name = "trinity_ui"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.6", features = ["csr"] }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["HtmlAudioElement"] }
gloo-timers = "0.2"
```

**`src/main.rs`**
```rust
use leptos::*;
mod components;
use components::main_menu::MainMenu;

fn main() {
    // Mount the app to the body.
    mount_to_body(move || view! { <App/> })
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <>
            <link rel="stylesheet" href="/style.css"/>
            <MainMenu/>
        </>
    }
}
```

**`src/components/mod.rs`**
```rust
pub mod main_menu;
```

**`src/components/main_menu.rs`**
```rust
use leptos::*;
use gloo_timers::callback::Timeout;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlAudioElement;

/// Helper to play a sound effect.
fn play_sfx(src: &str) {
    let audio = HtmlAudioElement::new_with_src(src).unwrap();
    // Allow the browser to start playing without user gesture restrictions
    let _ = audio.play();
}

/// Simple animation helper using CSS classes.
fn trigger_animation(el: web_sys::Element, class_name: &str, duration_ms: u32) {
    el.class_list().add_1(class_name).ok();
    Timeout::new(duration_ms, move || {
        el.class_list().remove_1(class_name).ok();
    })
    .forget();
}

#[component]
pub fn MainMenu(cx: Scope) -> impl IntoView {
    // Refs for DOM nodes we want to animate
    let start_btn = create_node_ref::<html::Button>(cx);
    let options_btn = create_node_ref::<html::Button>(cx);
    let exit_btn = create_node_ref::<html::Button>(cx);

    // Click handlers with sound + animation
    let on_start = move |_| {
        if let Some(btn) = start_btn.get() {
            trigger_animation(btn.clone().unchecked_into(), "pulse", 300);
        }
        play_sfx("/sfx/confirm.wav");
        // TODO: navigate to game scene
    };
    let on_options = move |_| {
        if let Some(btn) = options_btn.get() {
            trigger_animation(btn.clone().unchecked_into(), "pulse", 300);
        }
        play_sfx("/sfx/click.wav");
        // TODO: open settings overlay
    };
    let on_exit = move |_| {
        if let Some(btn) = exit_btn.get() {
            trigger_animation(btn.clone().unchecked_into(), "shake", 500);
        }
        play_sfx("/sfx/error.wav");
        // TODO: close the app / navigate away
    };

    view! { cx,
        <div class="menu-screen">
            <h1 class="title">"TRINITY AI OS"</h1>
            <div class="button-grid">
                <button node_ref=start_btn on:click=on_start class="menu-btn neon">
                    "START"
                </button>
                <button node_ref=options_btn on:click=on_options class="menu-btn neon">
                    "SETTINGS"
                </button>
                <button node_ref=exit_btn on:click=on_exit class="menu-btn neon">
                    "EXIT"
                </button>
            </div>

            // Background animated particles
            <canvas id="bg-canvas" class="bg-canvas"></canvas>
        </div>
    }
}
```

**`static/style.css`**
```css
/* ==== Core Layout ==== */
body, html {
    margin: 0;
    padding: 0;
    height: 100%;
    background:#0a0a0a;
    font-family:'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    overflow:hidden;
    color:#e0e0e0;
}

/* ==== Menu Screen ==== */
.menu-screen {
    position:relative;
    width:100%;
    height:100%;
    display:flex;
    flex-direction:column;
    align-items:center;
    justify-content:center;
    text-align:center;
    z-index:1;
}

.title {
    font-size:3rem;
    margin-bottom:2rem;
    color:#00ffcc;
    text-shadow:0 0 10px #00ffcc, 0 0 20px #00ffcc;
}

/* ==== Buttons ==== */
.button-grid {
    display:grid;
    grid-template-columns:1fr;
    gap:1.5rem;
    width:300px;
}
.menu-btn {
    padding:1rem 2rem;
    font-size:1.25rem;
    border:none;
    border-radius:8px;
    cursor:pointer;
    background:#111;
    color:#00ffcc;
    position:relative;
    overflow:hidden;
    transition:background 0.3s, transform 0.2s;
}
.menu-btn::after {
    content:"";
    position:absolute;
    inset:0;
    background:linear-gradient(45deg,
        rgba(0,255,204,.1) 0%,
        rgba(0,255,204,.3) 50%,
        rgba(0,255,204,.1) 100%);
    opacity:0;
    transition:opacity .4s;
}
.menu-btn:hover {
    background:#222;
    transform:translateY(-2px);
}
.menu-btn:hover::after { opacity:1; }

/* ==== Neon Glow Effect ==== */
.neon {
    box-shadow:
        0 0 5px #00ffcc,
        0 0 10px #00ffcc,
        0 0 20px #00ffcc,
        0 0 40px #00ffcc;
}

/* ==== Animations ==== */
@keyframes pulse {
    0% { transform:scale(1); }
    50% { transform:scale(1.07); }
    100% { transform:scale(1); }
}
.pulse { animation:pulse .3s ease-out; }

@keyframes shake {
    0%,100%{transform:translateX(0);}
    25%{transform:translateX(-8px);}
    75%{transform:translateX(8px);}
}
.shake { animation:shake .5s ease-in-out; }

/* ==== Background Canvas ==== */
.bg-canvas {
    position:absolute;
    inset:0;
    width:100%;
    height:100%;
    z-index:0;
}

/* ==== Particle System (JS) ==== */
```

**`static/bg.js`**
```js
// Simple particle field for the background canvas.
// Loaded automatically via <script> tag in index.html.

function initParticles() {
    const canvas = document.getElementById('bg-canvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    let particles = [];

    function resize() {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
    }
    resize();
    window.addEventListener('resize', resize);

    class Particle {
        constructor() {
            this.reset();
        }
        reset() {
            this.x = Math.random() * canvas.width;
            this.y = Math.random() * canvas.height;
            this.vx = (Math.random() - 0.5) * 0.3;
            this.vy = (Math.random() - 0.5) * 0.3;
            this.size = Math.random() * 2 + 1;
            this.alpha = Math.random() * 0.5 + 0.2;
        }
        update() {
            this.x += this.vx;
            this.y += this.vy;
            if (this.x < 0 || this.x > canvas.width ||
                this.y < 0 || this.y > canvas.height) {
                this.reset();
            }
        }
        draw() {
            ctx.fillStyle = `rgba(0,255,204,${this.alpha})`;
            ctx.beginPath();
            ctx.arc(this.x, this.y, this.size, 0, Math.PI * 2);
            ctx.fill();
        }
    }

    for (let i = 0; i < 150; i++) particles.push(new Particle());

    function animate() {
        ctx.clearRect(0,0,canvas.width,canvas.height);
        particles.forEach(p => { p.update(); p.draw(); });
        requestAnimationFrame(animate);
    }
    animate();
}
window.addEventListener('load', initParticles);
```

**`static/index.html`**
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <title>Trinity AI OS - Main Menu</title>
    <script type="module">
        import init from './pkg/trinity_ui.js';
        init();
    </script>
    <script src="/bg.js"></