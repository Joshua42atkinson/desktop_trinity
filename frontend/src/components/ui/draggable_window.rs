use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn DraggableWindow(
    #[prop(default = "Untitled".to_string())] title: String,
    #[prop(default = true)] initial_visible: bool,
    children: Children,
) -> impl IntoView {
    let (position, set_position) = signal((100.0, 100.0)); // x, y
    let (size, _set_size) = signal((600.0, 400.0)); // width, height
    let (dragging, set_dragging) = signal(false);
    let (visible, set_visible) = signal(initial_visible);

    if !visible.get() {
        return view! { <div class="hidden"></div> }.into_any();
    }

    view! {
        <div
            class="fixed z-50 bg-gray-900/90 backdrop-blur-md border border-cyan-500/30 rounded-lg shadow-2xl flex flex-col transition-opacity duration-200"
            style:left=move || format!("{}px", position.get().0)
            style:top=move || format!("{}px", position.get().1)
            style:width=move || format!("{}px", size.get().0)
            style:height=move || format!("{}px", size.get().1)
            on:mousemove=move |ev: MouseEvent| {
                if dragging.get() {
                    set_position.update(|(x, y)| {
                        *x += ev.movement_x() as f64;
                        *y += ev.movement_y() as f64;
                    });
                }
            }
            on:mouseup=move |_| set_dragging.set(false)
            on:mouseleave=move |_| set_dragging.set(false)
        >
            // Header Bar
            <div
                class="h-8 bg-gray-800/80 rounded-t-lg flex items-center justify-between px-3 cursor-move border-b border-white/10 select-none"
                on:mousedown=move |_| set_dragging.set(true)
            >
                <div class="flex items-center gap-2">
                    <div class="w-3 h-3 rounded-full bg-red-400 hover:bg-red-500 cursor-pointer"
                         on:click=move |_| set_visible.set(false)></div>
                    <div class="w-3 h-3 rounded-full bg-yellow-400 cursor-not-allowed"></div>
                    <div class="w-3 h-3 rounded-full bg-green-400 cursor-not-allowed"></div>
                    <span class="text-xs font-mono text-gray-300 ml-2">{title}</span>
                </div>
                <div class="text-xs text-gray-500">"HUD"</div>
            </div>

            // Content Area
            <div class="flex-1 overflow-auto p-4 text-gray-200 font-mono text-sm scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-transparent">
                {children()}
            </div>
            
            // Resize Handle (Corner)
            <div class="absolute bottom-0 right-0 w-4 h-4 cursor-nwse-resize bg-transparent hover:bg-white/10 rounded-br-lg"
                 // Simplified resize logic for now, just a visual indicator
                 >
                 <div class="absolute bottom-1 right-1 w-2 h-2 border-r-2 border-b-2 border-gray-500"></div>
            </div>
        </div>
    }.into_any()
}
