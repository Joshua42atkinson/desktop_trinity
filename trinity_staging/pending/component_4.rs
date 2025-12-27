**/home/joshua/antigravity/trinity_overnight_work/src/components/button.rs**
```rust
//! Reusable glass‑morphism button component for Trinity AI OS.
//! Dark theme with smooth hover animation.
//!
//! ```
//! use crate::components::button::GlassButton;
//! view! {
//!     <GlassButton on:click=move |_| log::info!("Clicked!")>
//!         "Press Me"
//!     </GlassButton>
//! }
//! ```

use leptos::*;
use web_sys::MouseEvent;

/// Props for the `GlassButton`.
#[derive(Clone, Debug, PartialEq)]
pub struct GlassButtonProps {
    /// Optional CSS class name(s) to extend styling.
    #[prop(optional)]
    pub class: Option<String>,

    /// Content of the button (text, icons, etc.).
    #[prop(default = "")]
    pub children: Children,
}

/// `GlassButton` component.
///
/// The button uses a backdrop‑filter based glass effect and animates
/// its background blur and scale on hover/focus.
#[component]
pub fn GlassButton(
    cx: Scope,
    /// Props (optional class, children)
    #[prop(into)] props: GlassButtonProps,
) -> impl IntoView {
    // Merge user‑provided classes with the component's base class.
    let class = move || {
        let mut cls = String::from("glass-button");
        if let Some(extra) = &props.class {
            cls.push(' ');
            cls.push_str(extra);
        }
        cls
    };

    view! { cx,
        <button
            class=class()
            on:click=move |ev: MouseEvent| {
                // Propagate the click event to any external handler.
                ev.stop_propagation();
            }
        >
            {props.children(cx)}
        </button>
    }
}
```

**/home/joshua/antigravity/trinity_overnight_work/src/components/button.css**
```css
/* Glass‑morphism button – dark theme */
.glass-button {
  --bg-color: rgba(30, 30, 35, 0.6);
  --border-color: rgba(255, 255, 255, 0.12);
  --text-color: #e5e7eb;
  --hover-bg: rgba(45, 45, 55, 0.8);
  --shadow: 0 4px 30px rgba(0, 0, 0, 0.5);

  appearance: none;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 0.6rem 1.2rem;
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-color);
  background: var(--bg-color);
  backdrop-filter: blur(10px) saturate(180%);
  -webkit-backdrop-filter: blur(10px) saturate(180%);
  box-shadow: var(--shadow);
  cursor: pointer;
  transition:
    background 0.25s ease,
    transform 0.15s ease,
    backdrop-filter 0.25s ease;
}

/* Hover / focus animation */
.glass-button:hover,
.glass-button:focus-visible {
  background: var(--hover-bg);
  transform: scale(1.03);
  backdrop-filter: blur(12px) saturate(200%);
}

/* Active (pressed) state */
.glass-button:active {
  transform: scale(0.98);
}
```

**/home/joshua/antigravity/trinity_overnight_work/src/lib.rs**
```rust
mod components;

pub use components::button::{GlassButton, GlassButtonProps};

fn main() {
    // Leptos app entry – ensure the CSS is loaded.
    leptos::mount_to_body(|cx| view! { cx,
        <style>{include_str!("components/button.css")}</style>

        <main class="app-container">
            <h1 style="color: #e5e7eb;">"Trinity AI OS"</h1>
            <GlassButton class=Some("my-special".to_string())>
                "Launch"
            </GlassButton>
        </main>
    });
}
```

**/home/joshua/antigravity/trinity_overnight_work/src/components/mod.rs**
```rust
pub mod button;
```