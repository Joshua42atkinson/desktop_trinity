```rust
// /home/joshua/antigravity/trinity_overnight_work/src/components/modal.rs

use leptos::*;
use crate::styles::modal_css::MODAL_CSS;

/// Props for the reusable Modal component.
#[derive(Clone, PartialEq)]
pub struct ModalProps {
    /// Whether the modal is open.
    pub open: Signal<bool>,
    /// Callback when the backdrop is clicked (usually to close).
    pub on_close: Callback<()>,
    /// Optional title displayed at the top of the modal.
    pub title: Option<String>,
    /// The content of the modal.
    pub children: Children,
}

/// Reusable glass‑morphism styled Modal component.
///
/// # Usage
///
/// ```rust
/// let (show, set_show) = create_signal(false);
///
/// view! {
///     <button on:click=move |_| set_show.set(true)>"Open"</button>
///     <Modal open=show on_close=Callback::new(move |_| set_show.set(false)) title="My Modal">
///         <p>"Hello from the modal!"</p>
///     </Modal>
/// }
/// ```
#[component]
pub fn Modal(cx: Scope, props: ModalProps) -> impl IntoView {
    // Insert the CSS only once per app.
    provide_context::<&'static str>(cx, MODAL_CSS);
    let ModalProps {
        open,
        on_close,
        title,
        children,
    } = props;

    view! { cx,
        // Backdrop
        <Show when=move || open.get() fallback=|| ()>
            <div class="modal-backdrop" on:click=move |_| on_close.call(())></div>

            // Modal dialog
            <div class="modal-dialog">
                <Show when=move || title.is_some() fallback=|| ()>
                    <header class="modal-header">
                        <h2>{title.unwrap()}</h2>
                        <button class="modal-close" aria-label="Close"
                            on:click=move |_| on_close.call(())>"✕"</button>
                    </header>
                </Show>

                <section class="modal-body">
                    {children(cx)}
                </section>
            </div>
        </Show>
    }
}
```

```rust
// /home/joshua/antigravity/trinity_overnight_work/src/styles/modal_css.rs

/// CSS string for the glass‑morphism dark themed Modal.
///
/// The constant is injected into the DOM by `provide_context` in the component.
pub const MODAL_CSS: &str = r#"
    /* Glassmorphism Dark Theme Modal */
    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(8px);
        z-index: 999;
        animation: fadeIn 0.3s ease-out forwards;
    }

    .modal-dialog {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%) scale(0.95);
        max-width: 90vw;
        width: 420px;
        background: rgba(30, 30, 30, 0.45);
        border-radius: 16px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(12px) saturate(180%);
        border: 1px solid rgba(255, 255, 255, 0.15);
        color: #e0e0e0;
        z-index: 1000;
        overflow: hidden;
        animation: slideIn 0.35s ease-out forwards;
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 1.2rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
        background: rgba(20, 20, 20, 0.25);
    }

    .modal-header h2 {
        margin: 0;
        font-size: 1.15rem;
        color: #fafafa;
    }

    .modal-close {
        background: transparent;
        border: none;
        color: #bbb;
        font-size: 1.2rem;
        cursor: pointer;
        transition: color 0.2s ease, transform 0.2s ease;
    }

    .modal-close:hover {
        color: #fff;
        transform: rotate(90deg);
    }

    .modal-body {
        padding: 1.2rem;
        max-height: 70vh;
        overflow-y: auto;
    }

    /* Animations */
    @keyframes fadeIn {
        from { opacity: 0; }
        to   { opacity: 1; }
    }

    @keyframes slideIn {
        from {
            transform: translate(-50%, -55%) scale(0.9);
            opacity: 0;
        }
        to {
            transform: translate(-50%, -50%) scale(1);
            opacity: 1;
        }
    }

    /* Hover effect for backdrop (clickable area) */
    .modal-backdrop:hover {
        background: rgba(0, 0, 0, 0.7);
    }
"#;
```