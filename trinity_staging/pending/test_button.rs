```rust
use leptos::*;
use leptos::html::button;

/// Simple button component with a hover effect.
///
/// # Props
/// * `label` – Text displayed inside the button.
/// * `on_click` – Callback executed when the button is clicked.
#[component]
pub fn HoverButton(
    cx: Scope,
    #[prop(into)] label: String,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
    // Create a signal to toggle the hover class
    let (is_hovered, set_is_hovered) = create_signal(cx, false);

    // Event handlers for mouse enter/leave
    let on_mouse_enter = move |_| set_is_hovered.set(true);
    let on_mouse_leave = move |_| set_is_hovered.set(false);

    // Click handler forwarding to the optional callback
    let click_handler = {
        let cb = on_click.clone();
        move |ev: MouseEvent| {
            if let Some(cb) = &cb {
                cb(ev);
            }
        }
    };

    view! { cx,
        <button
            class=move || {
                // Base styles + conditional hover style
                let base = "px-4 py-2 rounded transition-colors duration-200 \
                            bg-blue-600 text-white focus:outline-none";
                if is_hovered.get() {
                    format!("{} {}", base, "bg-blue-500")
                } else {
                    base.to_string()
                }
            }
            on:mouseenter=on_mouse_enter
            on:mouseleave=on_mouse_leave
            on:click=click_handler
        >
            {label}
        </button>
    }
}

// ---------- Example usage ----------
#[component]
fn App(cx: Scope) -> impl IntoView {
    let click_msg = create_rw_signal(cx, String::new());

    view! { cx,
        <HoverButton
            label="Click me"
            on_click=Callback::new(move |_| click_msg.set("Button clicked!".to_string()))
        />
        <p>{move || click_msg.get()}</p>
    }
}
```