use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col items-center justify-center bg-gray-900 text-white">
            <h1 class="text-4xl font-bold mb-4">"404 - Page Not Found"</h1>
            <p class="text-gray-400">"The page you are looking for does not exist."</p>
            <a href="/" class="mt-6 px-4 py-2 bg-blue-600 rounded hover:bg-blue-500 transition">"Go Home"</a>
        </div>
    }
}
