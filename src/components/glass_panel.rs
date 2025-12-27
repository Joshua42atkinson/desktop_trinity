use leptos::*;

/// A reusable Glassmorphism container component.
/// Usage: <GlassPanel> ... content ... </GlassPanel>
#[component]
pub fn GlassPanel(
    children: Children,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    view! {
        <div class=format!("relative overflow-hidden rounded-2xl p-8 md:p-10 bg-slate-800/40 backdrop-blur-xl border border-white/10 shadow-2xl {}", class)>
            {children()}
        </div>
    }
}