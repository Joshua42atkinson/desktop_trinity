//! Trinity Creative Studios Hub
//!
//! Main landing page for all creative production workflows.

use leptos::prelude::*;

#[component]
pub fn StudiosHub() -> impl IntoView {
    view! {
        <div class="min-h-screen p-8">
            // Hero Section
            <div class="max-w-6xl mx-auto mb-12">
                <div class="text-center mb-8">
                    <h1 class="text-5xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-rose-400 via-purple-500 to-cyan-400 mb-4">
                        "Creative Studios"
                    </h1>
                    <p class="text-xl text-gray-400">
                        "Your AI-powered production hub for content creation"
                    </p>
                </div>
            </div>

            // Studios Grid
            <div class="max-w-6xl mx-auto grid grid-cols-1 md:grid-cols-2 gap-8">
                // Video Studio
                <a href="/studios/video" class="group relative overflow-hidden bg-gradient-to-br from-red-900/30 to-red-950/50 border border-red-500/20 rounded-2xl p-8 hover:border-red-500/50 hover:shadow-lg hover:shadow-red-500/10 transition-all duration-300">
                    <div class="absolute top-0 right-0 w-32 h-32 bg-red-500/10 rounded-full blur-3xl group-hover:bg-red-500/20 transition-colors"></div>
                    <div class="relative z-10">
                        <div class="text-5xl mb-4">"🎬"</div>
                        <h2 class="text-2xl font-bold text-white mb-2">"Video Studio"</h2>
                        <p class="text-gray-400 mb-4">"YouTube scripts, thumbnails, and SEO planning"</p>
                        <div class="flex flex-wrap gap-2">
                            <span class="px-2 py-1 text-xs bg-red-500/20 text-red-300 rounded">"Scripts"</span>
                            <span class="px-2 py-1 text-xs bg-red-500/20 text-red-300 rounded">"Thumbnails"</span>
                            <span class="px-2 py-1 text-xs bg-red-500/20 text-red-300 rounded">"SEO"</span>
                        </div>
                    </div>
                </a>

                // Game Studio
                <a href="/studios/games" class="group relative overflow-hidden bg-gradient-to-br from-green-900/30 to-green-950/50 border border-green-500/20 rounded-2xl p-8 hover:border-green-500/50 hover:shadow-lg hover:shadow-green-500/10 transition-all duration-300">
                    <div class="absolute top-0 right-0 w-32 h-32 bg-green-500/10 rounded-full blur-3xl group-hover:bg-green-500/20 transition-colors"></div>
                    <div class="relative z-10">
                        <div class="text-5xl mb-4">"🎮"</div>
                        <h2 class="text-2xl font-bold text-white mb-2">"Game Studio"</h2>
                        <p class="text-gray-400 mb-4">"Android and Roblox game development"</p>
                        <div class="flex flex-wrap gap-2">
                            <span class="px-2 py-1 text-xs bg-green-500/20 text-green-300 rounded">"Android"</span>
                            <span class="px-2 py-1 text-xs bg-green-500/20 text-green-300 rounded">"Roblox"</span>
                            <span class="px-2 py-1 text-xs bg-green-500/20 text-green-300 rounded">"GDD"</span>
                        </div>
                    </div>
                </a>

                // Web Studio
                <a href="/studios/web" class="group relative overflow-hidden bg-gradient-to-br from-cyan-900/30 to-cyan-950/50 border border-cyan-500/20 rounded-2xl p-8 hover:border-cyan-500/50 hover:shadow-lg hover:shadow-cyan-500/10 transition-all duration-300">
                    <div class="absolute top-0 right-0 w-32 h-32 bg-cyan-500/10 rounded-full blur-3xl group-hover:bg-cyan-500/20 transition-colors"></div>
                    <div class="relative z-10">
                        <div class="text-5xl mb-4">"🌐"</div>
                        <h2 class="text-2xl font-bold text-white mb-2">"Web Studio"</h2>
                        <p class="text-gray-400 mb-4">"Website building and deployment"</p>
                        <div class="flex flex-wrap gap-2">
                            <span class="px-2 py-1 text-xs bg-cyan-500/20 text-cyan-300 rounded">"Sites"</span>
                            <span class="px-2 py-1 text-xs bg-cyan-500/20 text-cyan-300 rounded">"Components"</span>
                            <span class="px-2 py-1 text-xs bg-cyan-500/20 text-cyan-300 rounded">"Deploy"</span>
                        </div>
                    </div>
                </a>

                // Writing Studio
                <a href="/studios/writing" class="group relative overflow-hidden bg-gradient-to-br from-amber-900/30 to-amber-950/50 border border-amber-500/20 rounded-2xl p-8 hover:border-amber-500/50 hover:shadow-lg hover:shadow-amber-500/10 transition-all duration-300">
                    <div class="absolute top-0 right-0 w-32 h-32 bg-amber-500/10 rounded-full blur-3xl group-hover:bg-amber-500/20 transition-colors"></div>
                    <div class="relative z-10">
                        <div class="text-5xl mb-4">"📖"</div>
                        <h2 class="text-2xl font-bold text-white mb-2">"Writing Studio"</h2>
                        <p class="text-gray-400 mb-4">"Novels and edutainment narratives"</p>
                        <div class="flex flex-wrap gap-2">
                            <span class="px-2 py-1 text-xs bg-amber-500/20 text-amber-300 rounded">"Novels"</span>
                            <span class="px-2 py-1 text-xs bg-amber-500/20 text-amber-300 rounded">"Chapters"</span>
                            <span class="px-2 py-1 text-xs bg-amber-500/20 text-amber-300 rounded">"Characters"</span>
                        </div>
                    </div>
                </a>
            </div>

            // Quick Stats
            <div class="max-w-6xl mx-auto mt-12">
                <div class="bg-slate-800/30 border border-white/10 rounded-xl p-6">
                    <h3 class="text-lg font-semibold text-white mb-4">"Recent Activity"</h3>
                    <div class="text-gray-500 text-center py-8">
                        "No recent projects. Start creating in any studio above!"
                    </div>
                </div>
            </div>
        </div>
    }
}
