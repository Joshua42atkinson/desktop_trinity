use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum TooltipPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

#[component]
pub fn Tooltip(
    /// The content to show in the tooltip
    #[prop(into)]
    text: String,
    /// Position of the tooltip relative to the trigger
    #[prop(optional)]
    position: TooltipPosition,
    /// Optional keyboard shortcut to display
    #[prop(optional, into)]
    shortcut: Option<String>,
    /// The element that triggers the tooltip
    children: Children,
) -> impl IntoView {
    let (is_visible, set_is_visible) = signal(false);

    let position_class = match position {
        TooltipPosition::Top => "bottom-full left-1/2 -translate-x-1/2 mb-2",
        TooltipPosition::Bottom => "top-full left-1/2 -translate-x-1/2 mt-2",
        TooltipPosition::Left => "right-full top-1/2 -translate-y-1/2 mr-2",
        TooltipPosition::Right => "left-full top-1/2 -translate-y-1/2 ml-2",
    };

    let arrow_class = match position {
        TooltipPosition::Top => "top-full left-1/2 -translate-x-1/2 border-t-slate-800 border-x-transparent border-b-transparent",
        TooltipPosition::Bottom => "bottom-full left-1/2 -translate-x-1/2 border-b-slate-800 border-x-transparent border-t-transparent",
        TooltipPosition::Left => "left-full top-1/2 -translate-y-1/2 border-l-slate-800 border-y-transparent border-r-transparent",
        TooltipPosition::Right => "right-full top-1/2 -translate-y-1/2 border-r-slate-800 border-y-transparent border-l-transparent",
    };

    view! {
        <div
            class="relative inline-block"
            on:mouseenter=move |_| set_is_visible.set(true)
            on:mouseleave=move |_| set_is_visible.set(false)
            on:focus=move |_| set_is_visible.set(true)
            on:blur=move |_| set_is_visible.set(false)
        >
            {children()}

            <Show when=move || is_visible.get()>
                <div
                    class=format!("absolute z-50 pointer-events-none {}", position_class)
                    role="tooltip"
                >
                    <div class="px-3 py-2 bg-slate-800 text-white text-sm rounded-lg shadow-xl border border-white/10 whitespace-nowrap animate-fade-in">
                        <div class="flex items-center gap-2">
                            <span>{text.clone()}</span>
                            {shortcut.clone().map(|s| view! {
                                <kbd class="px-1.5 py-0.5 bg-white/10 rounded text-xs font-mono border border-white/20">
                                    {s}
                                </kbd>
                            })}
                        </div>
                    </div>
                    // Arrow
                    <div class=format!("absolute w-0 h-0 border-4 {}", arrow_class)></div>
                </div>
            </Show>
        </div>
    }
}
