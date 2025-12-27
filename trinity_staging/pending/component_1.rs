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
    /// Optional CSS class to extend styling.
    pub class: Option<String>,
    /// Disabled state.
    #[prop(optional, default = false)]
    pub disabled: bool,
}

/// Reusable glass‑morphism button with dark theme and hover animation.
///
/// # Example
///
/// ```rust
/// view! { cx,
///     <GlassButton on_click=Callback::new(|_| log::info!("clicked"))>
///         "Press me"
///     </GlassButton>
/// }
/// ```
#[component]
pub fn GlassButton(
    cx: Scope,
    #[prop(into)] children: Children,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] class: Option<String>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let classes = move || {
        let mut base = String::from("glass-button");
        if let Some(extra) = &class {
            base.push(' ');
            base.push_str(extra);
        }
        if disabled {
            base.push_str(" disabled");
        }
        base
    };

    view! { cx,
        <button
            class=classes()
            on:click=move |ev| {
                if !disabled {
                    if let Some(cb) = &on_click {
                        cb.call(ev);
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
/* /home/joshua/antigravity/trinity_overnight_work/src/styles/button.css */

.glass-button {
    /* Glassmorphism base */
    background: rgba(30, 30, 30, 0.45);
    backdrop-filter: blur(12px) saturate(180%);
    -webkit-backdrop-filter: blur(12px) saturate(180%);

    color: #e0e0e0;
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 0.75rem;
    padding: 0.6rem 1.2rem;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition:
        background-color 0.25s ease,
        box-shadow 0.25s ease,
        transform 0.15s ease;

    /* Dark theme specifics */
    box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.07);
}

/* Hover / focus animation */
.glass-button:hover:not(.disabled),
.glass-button:focus-visible:not(.disabled) {
    background: rgba(45, 45, 45, 0.55);
    box-shadow:
        0 4px 12px rgba(0, 0, 0, 0.4),
        inset 0 1px 3px rgba(255, 255, 255, 0.08);
    transform: translateY(-2px);
}

/* Active press effect */
.glass-button:active:not(.disabled) {
    background: rgba(55, 55, 55, 0.6);
    transform: translateY(0);
}

/* Disabled state */
.glass-button.disabled,
.glass-button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
    pointer-events: none;
}
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/components/mod.rs

pub mod button;
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/main.rs

use leptos::*;
mod components;
mod styles;

fn main() {
    // Initialize logging (optional)
    console_error_panic_hook::set_once();

    mount_to_body(|cx| view! { cx,
        <link rel="stylesheet" href="/styles/button.css"/>
        <components::button::GlassButton on_click=Callback::new(|_| log::info!("Clicked!"))>
            "Launch Trinity"
        </components::button::GlassButton>
    });
}
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/styles/mod.rs

pub const BUTTON_CSS: &str = include_str!("button.css");
```

```html
<!-- In your static assets folder (e.g., /static/styles/button.css) copy the CSS above -->
<link rel="stylesheet" href="/static/styles/button.css"/>
```

*All files are placed under `/home/joshua/antigravity/trinity_overnight_work/` as requested.*