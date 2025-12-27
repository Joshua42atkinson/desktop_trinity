```rust
// /home/joshua/antigravity/trinity_overnight_work/src/components/glass_button.rs

use leptos::*;
use crate::styles::glass_button::GLASS_BUTTON_CSS;

/// Re‑usable, glass‑morphic button component.
///
/// # Props
/// * `label` – Text displayed inside the button.
/// * `on_click` – Callback executed when the button is clicked.
/// * `disabled` – Optional flag to disable the button.
#[component]
pub fn GlassButton(
    #[prop(into)] label: String,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, default = false)] disabled: bool,
) -> impl IntoView {
    // Load CSS once
    static STYLE_LOADED: OnceCell<()> = OnceCell::new();
    STYLE_LOADED.get_or_init(|| {
        let style = document().create_element("style").unwrap();
        style.set_inner_html(GLASS_BUTTON_CSS);
        document()
            .head()
            .expect("no <head>")
            .append_child(&style)
            .expect("failed to append style");
    });

    view! {
        <button
            class=move || if disabled { "glass-button disabled" } else { "glass-button" }
            on:click=move |ev| {
                if let Some(cb) = &on_click {
                    cb(ev);
                }
            }
            disabled=disabled
        >
            {label}
        </button>
    }
}
```

```css
/* /home/joshua/antigravity/trinity_overnight_work/src/styles/glass_button.css */

.glass-button {
    position: relative;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    color: #e0e0e0;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    box-shadow: inset 0 0 0 1px rgba(255,255,255,0.04),
                0 8px 32px 0 rgba(0,0,0,0.37);
    cursor: pointer;
    transition:
        background 0.3s ease,
        transform 0.2s ease,
        box-shadow 0.3s ease;
    user-select: none;
}

.glass-button::before {
    content: "";
    position: absolute;
    inset: -1px;
    border-radius: inherit;
    background: linear-gradient(135deg, rgba(255,255,255,0.15), transparent);
    opacity: 0;
    transition: opacity 0.3s ease;
}

.glass-button:hover {
    background: rgba(255, 255, 255, 0.12);
    transform: translateY(-2px);
    box-shadow: inset 0 0 0 1px rgba(255,255,255,0.06),
                0 12px 40px 0 rgba(0,0,0,0.45);
}

.glass-button:hover::before {
    opacity: 0.6;
}

.glass-button:active {
    transform: translateY(0);
    background: rgba(255, 255, 255, 0.15);
}

.glass-button.disabled,
.glass-button[disabled] {
    cursor: not-allowed;
    opacity: 0.5;
    pointer-events: none;
}
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/styles/mod.rs

pub mod glass_button;

pub const GLASS_BUTTON_CSS: &str = include_str!("glass_button.css");
```

```toml
# Add to Cargo.toml if not already present
[dependencies]
leptos = { version = "0.6", features = ["csr"] }
once_cell = "1.19"
```

```rust
// Example usage in a page component:
//
// use crate::components::glass_button::GlassButton;
//
// #[component]
// fn DemoPage() -> impl IntoView {
//     let click_cb = Callback::new(|_: MouseEvent| {
//         web_sys::console::log_1(&"Glass button clicked!".into());
//     });
//
//     view! {
//         <div style="display: flex; gap: 1rem; padding: 2rem; background:#111;">
//             <GlassButton label="Start".to_string() on_click=Some(click_cb) />
//         </div>
//     }
// }
```