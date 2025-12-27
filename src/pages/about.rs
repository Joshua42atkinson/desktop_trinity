use leptos::*;
use crate::components::glass_panel::GlassPanel;

#[component]
pub fn About() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto space-y-12 animate-fade-in">
            // Header
            <div class="text-center space-y-4">
                <h1 class="text-4xl md:text-5xl font-bold text-white">"Meet the " <span class="text-cyan-400">"Designer"</span></h1>
                <div class="h-1 w-24 bg-gradient-to-r from-cyan-500 to-blue-600 mx-auto rounded-full"></div>
            </div>

            // Content Grid
            <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                // Sidebar / Contact Info
                <div class="md:col-span-1 space-y-6">
                    <GlassPanel class="text-center space-y-4">
                        <div class="w-32 h-32 mx-auto rounded-full bg-slate-700 border-2 border-cyan-500 overflow-hidden relative">
                            // Placeholder for your headshot
                            <div class="absolute inset-0 flex items-center justify-center text-4xl">"JA"</div>
                        </div>
                        <div>
                            <h3 class="text-xl font-bold text-white">"Joshua Atkinson"</h3>
                            <p class="text-cyan-400 text-sm">"Instructional Architect"</p>
                        </div>
                        <div class="text-left space-y-2 text-sm text-slate-300 pt-4 border-t border-white/10">
                            <p><span class="text-slate-500 block text-xs uppercase tracking-widest">"Email"</span> "joshua42atkinson@gmail.com"</p>
                            <p><span class="text-slate-500 block text-xs uppercase tracking-widest">"Location"</span> "Houlton, Maine"</p>
                        </div>
                    </GlassPanel>
                </div>

                // Main Bio
                <div class="md:col-span-2 space-y-6">
                    <GlassPanel>
                        <div class="prose prose-invert max-w-none">
                            <p class="text-lg text-gray-200 leading-relaxed">
                                "I'm a Marine Corps veteran with a passion for fostering a love of learning through innovative educational practices. I bring a unique perspective to education, valuing both playful curiosity and focused engagement."
                            </p>
                            <p class="text-gray-400">
                                "From pastor's kid to Marine Gunnery Sergeant, from biker bars to political boardrooms, my life has been a kaleidoscope of experiences. After trading in camo for cables as an IT installer, I launched a trucking company, tossed pizzas, was a bartender, and worked in non-profit community development."
                            </p>
                            <p class="text-gray-400">
                                "But beneath the public service and business suits is an intense desire to develop challenging and playful learning experiences for myself and others."
                            </p>
                        </div>
                    </GlassPanel>
                </div>
            </div>
        </div>
    }
}