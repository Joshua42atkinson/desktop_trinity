```rust
// /home/joshua/antigravity/trinity_overnight_work/src/components/button.rs

use leptos::*;
use web_sys::MouseEvent;

/// Props for the `GlassButton` component.
#[derive(Clone, PartialEq)]
pub struct GlassButtonProps {
    /// Button label text.
    pub children: Children,
    /// Optional click handler.
    pub on_click: Option<Callback<MouseEvent>>,
    /// Optional disabled state.
    pub disabled: bool,
    /// Optional custom class for extra styling.
    pub class: Option<String>,
}

/// Reusable glass‑morphism button with dark theme and hover animation.
///
/// # Example
///
/// ```rust
/// view! {
///     <GlassButton on_click=move |_| log::info!("clicked!")>
///         "Click Me"
///     </GlassButton>
/// }
/// ```
#[component]
pub fn GlassButton(
    cx: Scope,
    #[prop(optional)] children: Children,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, default = false)] disabled: bool,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let base_class = "glass-button".to_string();
    let classes = match class {
        Some(extra) => format!("{} {}", base_class, extra),
        None => base_class,
    };

    view! { cx,
        <button
            class=classes
            on:click=move |ev| {
                if !disabled {
                    if let Some(cb) = &on_click {
                        cb(ev);
                    }
                }
            }
            disabled=disabled
        >
            {children(cx)}
        </button>
    }
}
```

```css
/* /home/joshua/antigravity/trinity_overnight_work/src/components/button.css */

.glass-button {
    /* Glassmorphism base */
    background: rgba(30, 30, 40, 0.45);
    backdrop-filter: blur(12px) saturate(180%);
    -webkit-backdrop-filter: blur(12px) saturate(180%);
    border-radius: 0.75rem;
    border: 1px solid rgba(255, 255, 255, 0.18);
    color: #e0e0e0;
    padding: 0.6rem 1.2rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition:
        transform 0.2s ease,
        box-shadow 0.2s ease,
        background-color 0.2s ease;
    outline: none;
}

/* Hover & focus animation */
.glass-button:hover:not(:disabled),
.glass-button:focus-visible:not(:disabled) {
    transform: translateY(-2px);
    box-shadow:
        0 8px 32px rgba(0, 0, 0, 0.4),
        inset 0 0 0 1px rgba(255, 255, 255, 0.12);
    background: rgba(45, 45, 60, 0.55);
}

/* Active press effect */
.glass-button:active:not(:disabled) {
    transform: translateY(0);
    box-shadow:
        0 4px 16px rgba(0, 0, 0, 0.3),
        inset 0 0 0 1px rgba(255, 255, 255, 0.08);
}

/* Disabled state */
.glass-button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
}
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/lib.rs

mod components;

pub use components::button::*;
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/components/mod.rs

pub mod button;
```

```toml
# /home/joshua/antigravity/trinity_overnight_work/Cargo.toml

[package]
name = "trinity_overnight_work"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.6", features = ["csr"] }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["MouseEvent"] }
```

```rust
// Example usage in a page component
// /home/joshua/antigravity/trinity_overnight_work/src/pages/home.rs

use leptos::*;
use crate::components::button::GlassButton;

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    view! { cx,
        <div style="display:flex; justify-content:center; align-items:center; height:100vh;">
            <GlassButton
                on_click=move |_| log::info!("Glass button clicked!")
                class="my-custom-class"
            >
                "Launch Trinity"
            </GlassButton>
        </div>
    }
}
```

```rust
// Ensure the CSS is included in your index.html or via a build step.
// Example snippet for index.html:
//
// <link rel="stylesheet" href="/components/button.css">
```