use leptos::prelude::*;

#[component]
pub fn LoadingSpinner(
    /// Optional message to display below the spinner
    #[prop(optional, into)]
    message: Option<String>,
    /// Size of the spinner (sm, md, lg)
    #[prop(optional)]
    size: Option<String>,
) -> impl IntoView {
    let size = size.unwrap_or_else(|| "md".to_string());

    let (spinner_size, message_class) = match size.as_str() {
        "sm" => ("w-6 h-6", "text-xs"),
        "lg" => ("w-16 h-16", "text-lg"),
        _ => ("w-10 h-10", "text-sm"), // md is default
    };

    view! {
        <div class="flex flex-col items-center justify-center gap-3">
            <div class=format!("relative {}", spinner_size)>
                // Outer ring
                <div class="absolute inset-0 rounded-full border-4 border-purple-500/20"></div>
                // Spinning gradient ring
                <div class="absolute inset-0 rounded-full border-4 border-transparent border-t-purple-500 border-r-cyan-500 animate-spin"></div>
                // Inner glow
                <div class="absolute inset-2 rounded-full bg-gradient-to-tr from-purple-500/20 to-cyan-500/20 animate-pulse"></div>
            </div>

            {message.map(|msg| view! {
                <p class=format!("text-slate-300 font-medium {}", message_class)>
                    {msg}
                </p>
            })}
        </div>
    }
}
