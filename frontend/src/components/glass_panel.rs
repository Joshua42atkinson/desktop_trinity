use leptos::prelude::*;

#[component]
pub fn GlassPanel(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let base_class = "relative overflow-hidden rounded-2xl p-8 md:p-10 bg-slate-800/40 backdrop-blur-xl border border-white/10 shadow-2xl";
    let combined_class = if class.is_empty() {
        base_class.to_string()
    } else {
        format!("{} {}", base_class, class)
    };

    view! {
        <div class=combined_class>
            {children()}
        </div>
    }
}
