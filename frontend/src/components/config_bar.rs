use leptos::prelude::*;

#[component]
pub fn ConfigBar(
    api_url: ReadSignal<String>,
    set_api_url: WriteSignal<String>,
    api_key: ReadSignal<String>,
    set_api_key: WriteSignal<String>,
    user_id: ReadSignal<String>,
    set_user_id: WriteSignal<String>,
    on_load: impl Fn() + 'static,
    on_sync: impl Fn() + 'static,
    loading: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="bg-surface border border-border rounded-lg p-4 mb-5 flex flex-col sm:flex-row sm:flex-wrap gap-3 sm:items-end">
            <div class="w-full sm:w-auto">
                <label class="text-dim text-xs block mb-1">"API URL"</label>
                <input
                    class="bg-bg border border-border text-text px-3 py-2.5 rounded text-sm font-mono w-full sm:w-64"
                    prop:value=move || api_url.get()
                    on:input=move |e| {
                        use wasm_bindgen::JsCast;
                        let v = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                        set_api_url.set(v);
                    }
                />
            </div>
            <div class="w-full sm:w-auto">
                <label class="text-dim text-xs block mb-1">"API Key"</label>
                <input
                    type="password"
                    class="bg-bg border border-border text-text px-3 py-2.5 rounded text-sm font-mono w-full sm:w-72"
                    prop:value=move || api_key.get()
                    on:input=move |e| {
                        use wasm_bindgen::JsCast;
                        let v = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                        set_api_key.set(v);
                    }
                />
            </div>
            <div class="w-full sm:w-auto">
                <label class="text-dim text-xs block mb-1">"User ID"</label>
                <input
                    class="bg-bg border border-border text-text px-3 py-2.5 rounded text-sm font-mono w-full sm:w-72"
                    prop:value=move || user_id.get()
                    on:input=move |e| {
                        use wasm_bindgen::JsCast;
                        let v = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                        set_user_id.set(v);
                    }
                />
            </div>
            <div class="flex gap-2 w-full sm:w-auto">
                <button
                    class="bg-accent text-bg px-5 min-h-[44px] rounded font-bold text-sm disabled:opacity-50 flex-1 sm:flex-none"
                    on:click=move |_| on_load()
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Loading..." } else { "Load" }}
                </button>
                <button
                    class="bg-border text-text px-5 min-h-[44px] rounded font-bold text-sm flex-1 sm:flex-none"
                    on:click=move |_| on_sync()
                >
                    "Sync Now"
                </button>
            </div>
        </div>
    }
}
