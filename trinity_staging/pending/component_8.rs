```rust
// /home/joshua/antigravity/trinity_overnight_work/src/components/glass_button.rs

use leptos::*;
use web_sys::MouseEvent;

/// Props for the `GlassButton` component.
#[derive(Clone, PartialEq)]
pub struct GlassButtonProps {
    /// Text displayed inside the button.
    pub label: String,
    /// Optional callback when the button is clicked.
    pub on_click: Option<Callback<MouseEvent>>,
    /// Optional additional CSS classes.
    pub class: Option<String>,
}

/// Reusable glass‑morphism styled button with dark theme and hover animation.
///
/// # Example
///
/// ```rust
/// view! {
///     <GlassButton label="Launch".to_string()
///                 on_click=Some(move |_| log::info!("Clicked")) />
/// }
/// ```
#[component]
pub fn GlassButton(
    cx: Scope,
    #[prop(into)] label: String,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let classes = move || {
        let mut base = "glass-button".to_string();
        if let Some(extra) = &class {
            base.push(' ');
            base.push_str(extra);
        }
        base
    };

    view! { cx,
        <button class=classes on:click=move |ev| {
            if let Some(cb) = &on_click {
                cb(ev);
            }
        }>
            {label}
        </button>
    }
}
```

```css
/* /home/joshua/antigravity/trinity_overnight_work/src/styles/glass_button.css */

/* Glassmorphism dark‑theme button */
.glass-button {
    position: relative;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 600;
    color: #e0e0e0;
    background: rgba(30, 30, 30, 0.45);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 0.75rem;
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    cursor: pointer;
    overflow: hidden;
    transition: background 0.3s ease, transform 0.2s ease, box-shadow 0.3s ease;
}

/* Hover & focus animation */
.glass-button::before {
    content: "";
    position: absolute;
    inset: 0;
    background: linear-gradient(135deg,
        rgba(255, 255, 255, 0.08),
        rgba(255, 255, 255, 0) 70%);
    opacity: 0;
    transition: opacity 0.3s ease;
}

.glass-button:hover,
.glass-button:focus-visible {
    background: rgba(45, 45, 45, 0.55);
    transform: translateY(-2px);
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.4);
}

.glass-button:hover::before,
.glass-button:focus-visible::before {
    opacity: 1;
}

/* Active state */
.glass-button:active {
    transform: translateY(0);
    background: rgba(55, 55, 55, 0.65);
}
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/components/mod.rs

pub mod glass_button;
```

```toml
# Add to Cargo.toml if not already present
[dependencies]
leptos = { version = "0.6", features = ["csr"] }
web-sys = { version = "0.3", features = ["MouseEvent"] }
```

```rust
// Example usage in a page component
// /home/joshua/antigravity/trinity_overnight_work/src/pages/home.rs

use leptos::*;
use crate::components::glass_button::GlassButton;

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    view! { cx,
        <div style="display:flex; justify-content:center; align-items:center; height:100vh; background:#111;">
            <GlassButton
                label="Launch Trinity".to_string()
                on_click=Some(move |_| log::info!("Trinity launched"))
                class=Some("my-custom-class".to_string())
            />
        </div>
    }
}
```

```rust
// Register the stylesheet in your main entry point
// /home/joshua/antigravity/trinity_overnight_work/src/main.rs

use leptos::*;
mod components;
mod pages;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx,
        <link rel="stylesheet" href="/styles/glass_button.css"/>
        <pages::home::Home/>
    });
}
```