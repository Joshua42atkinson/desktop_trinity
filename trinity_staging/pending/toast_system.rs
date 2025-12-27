**src/main.rs**
```rust
use leptos::*;
use leptos_router::*;
use trinity_toast::ToastProvider;

fn main() {
    // mount the app
    mount_to_body(move || view! { <App/> })
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <ToastProvider>
            // Example UI to trigger toasts
            <div class="demo">
                <button on:click=move |_| toast::push_success("Operation completed!")>"Success"</button>
                <button on:click=move |_| toast::push_error("Something went wrong.")>"Error"</button>
                <button on:click=move |_| toast::push_warning("Be careful…")>"Warning"</button>
                <button on:click=move |_| toast::push_info("Just so you know…")>"Info"</button>
                <button on:click=move |_| toast::push_achievement(
                    "First Blood",
                    "Defeat your first enemy."
                )>"Achievement"</button>
            </div>
        </ToastProvider>
    }
}
```

**src/lib.rs**
```rust
pub mod toast;
```

**src/toast.rs**
```rust
use leptos::*;
use std::rc::Rc;
use web_sys::{HtmlDivElement, HtmlAudioElement};

/// Configuration for a single toast.
#[derive(Clone)]
pub struct ToastConfig {
    pub id: usize,
    pub kind: ToastKind,
    pub title: String,
    pub message: String,
    pub duration_ms: u32,
    pub sound_url: Option<String>,
}

/// Different toast kinds.
#[derive(Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Warning,
    Info,
    Achievement { subtitle: String },
}

impl ToastKind {
    fn icon(&self) -> &'static str {
        match self {
            ToastKind::Success => include_str!("icons/success.svg"),
            ToastKind::Error => include_str!("icons/error.svg"),
            ToastKind::Warning => include_str!("icons/warning.svg"),
            ToastKind::Info => include_str!("icons/info.svg"),
            ToastKind::Achievement { .. } => include_str!("icons/achievement.svg"),
        }
    }

    fn sound(&self) -> Option<&'static str> {
        match self {
            ToastKind::Success => Some(include_str!("sounds/success.mp3")),
            ToastKind::Error => Some(include_str!("sounds/error.mp3")),
            ToastKind::Warning => Some(include_str!("sounds/warning.mp3")),
            ToastKind::Info => Some(include_str!("sounds/info.mp3")),
            ToastKind::Achievement { .. } => Some(include_str!("sounds/achievement.mp3")),
        }
    }

    fn css_class(&self) -> &'static str {
        match self {
            ToastKind::Success => "toast-success",
            ToastKind::Error => "toast-error",
            ToastKind::Warning => "toast-warning",
            ToastKind::Info => "toast-info",
            ToastKind::Achievement { .. } => "toast-achievement",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            ToastKind::Success => "Success",
            ToastKind::Error => "Error",
            ToastKind::Warning => "Warning",
            ToastKind::Info => "Info",
            ToastKind::Achievement { .. } => "Achievement Unlocked!",
        }
    }
}

/// Global toast context.
#[derive(Clone)]
pub struct ToastContext {
    pub push: Rc<dyn Fn(ToastConfig)>,
    pub close: Rc<dyn Fn(usize)>,
}

type ToastSignal = RwSignal<Vec<ToastConfig>>;

/// Provider component that holds the toast stack and renders them.
#[component]
pub fn ToastProvider(cx: Scope, #[prop(optional)] children: Children) -> impl IntoView {
    let toasts: ToastSignal = create_rw_signal(cx, vec![]);
    let next_id = Rc::new(Cell::new(0usize));

    // push implementation
    let push = {
        let toasts = toasts.clone();
        let next_id = next_id.clone();
        Rc::new(move |mut cfg: ToastConfig| {
            let id = next_id.get();
            next_id.set(id + 1);
            cfg.id = id;
            // default duration if not set
            if cfg.duration_ms == 0 {
                cfg.duration_ms = match cfg.kind {
                    ToastKind::Achievement { .. } => 8000,
                    _ => 5000,
                };
            }
            // attach sound url if static asset provided
            cfg.sound_url = cfg.kind.sound().map(|s| s.to_string());

            toasts.update(move |list| {
                list.push(cfg);
                // keep only the last 5
                if list.len() > 5 {
                    list.drain(0..list.len() - 5);
                }
            });
        }) as Rc<dyn Fn(ToastConfig)>
    };

    // close implementation
    let close = {
        let toasts = toasts.clone();
        Rc::new(move |id: usize| {
            toasts.update(|list| list.retain(|t| t.id != id));
        })
    };

    provide_context(cx, ToastContext { push, close });

    view! { cx,
        <div class="toast-root">
            // render the stack
            {move || {
                let items = toasts.get();
                view! { cx,
                    <For
                        each=move || items.clone().into_iter()
                        key=|t| t.id
                        children=move |cx, toast: ToastConfig| {
                            view!{ cx, <ToastItem config=toast/> }
                        }/>
                }
            }}
        </div>
        {children(cx)}
    }
}

/// Individual toast component.
#[component]
fn ToastItem(
    cx: Scope,
    #[prop(into)] config: ToastConfig,
) -> impl IntoView {
    let ctx = use_context::<ToastContext>(cx).expect("ToastContext missing");
    let progress = create_signal(cx, 0.0);
    let expanded = create_signal(cx, false);
    let node_ref: NodeRef<HtmlDivElement> = create_node_ref(cx);

    // start timer
    let duration = config.duration_ms as f64;
    let tick = move || {
        let step = 50.0 / duration; // update every 50ms
        set_timeout_with_handle(
            move || {
                progress.update(|p| *p += step);
                if *progress.get() < 1.0 {
                    tick();
                } else {
                    (ctx.close)(config.id);
                }
            },
            50,
        )
        .expect("setTimeout");
    };
    // start on mount
    spawn_local(async move { tick(); });

    // play sound
    if let Some(url) = config.sound_url.clone() {
        let audio: HtmlAudioElement = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("audio")
            .unwrap()
            .unchecked_into();
        audio.set_src(&url);
        let _ = audio.play();
    }

    // slide‑in animation
    on_mount(cx, move || {
        if let Some(div) = node_ref.get() {
            div.class_list().add_1("slide-in").unwrap();
        }
    });

    view! { cx,
        <div class=format!("toast {}", config.kind.css_class()) ref=node_ref>
            // icon
            <div class="toast-icon" inner_html=config.kind.icon()/>
            // content
            <div class="toast-content">
                <div class="toast-header"
                     on:click=move |_| expanded.update(|e| *e = !*e)>
                    <strong>{config.kind.title()}</strong>
                    <span class="toast-title">{&config.title}</span>
                </div>
                // message (collapsible)
                {move || if *expanded.get() {
                    view!{ cx, <p class="toast-message">{&config.message}</p> }.into_view(cx)
                } else {
                    view!{ cx,
                        <p class="toast-message toast-collapsed">
                            {if config.message.len() > 80 {
                                format!("{}…", &config.message[..80])
                            } else {
                                config.message.clone()
                            }}
                        </p>
                    }.into_view(cx)
                }}
            </div>
            // close button
            <button class="toast-close"
                    on:click=move |_| (ctx.close)(config.id)>"✕"</button>

            // progress bar
            <div class="toast-progress">
                <div class="toast-progress-inner"
                     style=format!("width: {}%;", (*progress.get() * 100.0))></div>
            </div>
        </div>
    }
}

/* -------------------------------------------------------------------------- */
/* Helper functions for external use                                          */
/* -------------------------------------------------------------------------- */

/// Shortcut helpers that can be called from anywhere in the app.
pub mod toast {
    use super::*;
    use leptos::use_context;

    fn push(kind: ToastKind, title: impl Into<String>, message: impl Into<String>) {
        let ctx = use_context::<ToastContext>()
            .expect("ToastContext not found. Did you forget to wrap your app with <ToastProvider/>?");
        (ctx.push)(ToastConfig {
            id: 0,
            kind,
            title: title.into(),
            message: message.into(),
            duration_ms: 0,
            sound_url: None,
        });
    }

    pub fn push_success(message: impl Into<String>) {
        push(Toast