**File:** `/home/joshua/antigravity/trinity_overnight_work/src/components/button.rs`
```rust
//! Glassmorphism styled button component for Trinity AI OS.
//! Dark theme with smooth hover animation.
//!
//! # Usage
//! ```
//! use crate::components::button::GlassButton;
//! view! {
//!     <GlassButton on:click=|_| log::info!("clicked!")>
//!         "Press Me"
//!     </GlassButton>
//! }
//! ```

use leptos::*;
use web_sys::MouseEvent;

/// Props for the `GlassButton` component.
#[derive(Clone, PartialEq)]
pub struct GlassButtonProps {
    /// Optional CSS class name(s) to extend styling.
    pub class: Option<String>,
    /// The button's children (usually text or icons).
    #[prop(optional, into)]
    pub children: Children,
    /// Click event handler.
    #[prop(optional, into)]
    pub on_click: Option<Callback<MouseEvent>>,
}

#[component]
pub fn GlassButton(
    cx: Scope,
    #[prop(into)] class: Option<String>,
    #[prop(optional, into)] children: Children,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
    // Merge user‑provided classes with the component's base class.
    let combined_class = move || {
        let mut classes = vec!["glass-button"];
        if let Some(ref extra) = class {
            classes.push(extra);
        }
        classes.join(" ")
    };

    view! { cx,
        <button
            class=combined_class()
            on:click=move |ev| {
                if let Some(cb) = &on_click {
                    cb.call(ev);
                }
            }
        >
            {children(cx)}
        </button>
    }
}
```

---

**File:** `/home/joshua/antigravity/trinity_overnight_work/src/components/button.css`
```css
/* Glassmorphism button – dark theme */
.glass-button {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.75rem 1.5rem;
    font-size: 0.95rem;
    font-weight: 600;
    color: #e0e0e0;
    background: rgba(30, 30, 35, 0.45);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 0.75rem;
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    cursor: pointer;
    transition:
        background 0.3s ease,
        transform 0.2s ease,
        box-shadow 0.3s ease;
    overflow: hidden;
}

/* Subtle inner glow */
.glass-button::before {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: linear-gradient(
        135deg,
        rgba(255, 255, 255, 0.08),
        transparent
    );
    opacity: 0;
    transition: opacity 0.3s ease;
}

/* Hover state */
.glass-button:hover {
    background: rgba(45, 45, 55, 0.6);
    transform: translateY(-2px);
    box-shadow:
        0 8px 30px rgba(0, 0, 0, 0.4),
        inset 0 0 20px rgba(255, 255, 255, 0.08);
}

/* Reveal inner glow on hover */
.glass-button:hover::before {
    opacity: 1;
}

/* Active (pressed) state */
.glass-button:active {
    transform: translateY(0);
    background: rgba(55, 55, 65, 0.7);
}
```

---

**File:** `/home/joshua/antigravity/trinity_overnight_work/src/components/mod.rs`
```rust
pub mod button;

pub use button::GlassButton;
```

*Make sure to import the CSS in your main HTML or entry point, e.g.:*

```html
<link rel="stylesheet" href="/static/components/button.css">
```