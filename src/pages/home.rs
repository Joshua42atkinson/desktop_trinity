use leptos::*;
use crate::components::glass_panel::GlassPanel;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div class="max-w-5xl mx-auto w-full space-y-10 animate-fade-in">

            // --- Header Section ---
            <div class="text-center space-y-6">
                <div class="inline-flex items-center px-3 py-1 rounded-full bg-slate-800/60 backdrop-blur-md border border-cyan-900/50 text-cyan-400 text-xs font-bold tracking-widest uppercase shadow-lg">
                    <span class="w-2 h-2 bg-cyan-400 rounded-full mr-2 animate-pulse"></span>
                    "LDT Portfolio // Fall 2025"
                </div>

                <h1 class="text-5xl md:text-7xl font-extrabold tracking-tight leading-tight text-white">
                    "Joshua "
                    <span class="text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-blue-500">
                        "Atkinson"
                    </span>
                </h1>

                <p class="text-lg md:text-xl text-slate-400 max-w-2xl mx-auto leading-relaxed">
                    "Bridging the gap between "
                    <span class="text-cyan-200 font-semibold">"learning science"</span>
                    " and "
                    <span class="text-cyan-200 font-semibold">"technical architecture"</span>
                    "."
                </p>
            </div>

            // --- "The Hook" - Glass Panel with Bio ---
            <GlassPanel class="border-l-4 border-l-cyan-500">
                // Decorative Quotes
                <div class="absolute top-4 left-4 text-7xl text-cyan-500/10 font-serif select-none">"\""</div>

                <div class="relative z-10 space-y-6 text-left">
                    <p class="text-lg md:text-xl text-gray-200 leading-relaxed font-light">
                        "I bring a unique perspective to education, valuing both "
                        <strong class="text-white font-semibold">"playful curiosity"</strong>
                        " and "
                        <strong class="text-white font-semibold">"focused engagement"</strong>
                        ". My goal is simple: to develop challenging learning experiences that transform lives."
                    </p>

                    <hr class="border-gray-700/50" />

                    <div class="text-gray-400 text-base md:text-lg leading-relaxed space-y-4">
                        <p>
                            "My design philosophy is informed by a kaleidoscope of experiences. From pastor's kid to "
                            <span class="text-cyan-400 font-medium">"Marine Gunnery Sergeant"</span>
                            "."
                        </p>
                        <p>
                            "After trading in camo for cables, I launched a trucking company, tossed pizzas, and worked in non-profit community development. Beneath the public service and business suits is an intense desire to build dynamic solutions for student success."
                        </p>
                    </div>
                </div>
            </GlassPanel>

            // --- Footer ---
            <div class="text-center text-slate-600 text-xs pt-4 font-mono">
                "Designed with Rust-Stack Philosophy // Built for Impact"
            </div>
        </div>
    }
}