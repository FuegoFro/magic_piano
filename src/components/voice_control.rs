use leptos::{component, view, Callable, Callback, IntoView, Signal, SignalGet};

#[component]
pub fn VoiceControl(
    #[prop(into)] name: String,
    #[prop(into)] mute: Signal<bool>,
    #[prop(into)] solo: Signal<bool>,
    #[prop(into)] any_solo: Signal<bool>,
    #[prop(into)] on_toggle_mute: Callback<()>,
    #[prop(into)] on_toggle_solo: Callback<()>,
) -> impl IntoView {
    let muted = move || mute.get() || (!solo.get() && any_solo.get());

    view! {
        <div class="flex flex-col items-center p-4 border border-black border-solid rounded-sm">
            <p class:text-red-600=muted>{name}</p>
            <div class="flex flex-row space-x-1">
                <button
                    class="border border-black rounded-sm px-1"
                    class:bg-red-600=mute on:click=move |_| on_toggle_mute.call(())>
                    Mute
                </button>
                <button
                    class="border border-black rounded-sm px-1"
                    class:bg-sky-600=solo on:click=move |_| on_toggle_solo.call(())>
                    Solo
                </button>
            </div>
            <p>"Volume (doesn't work)"</p>
            <input type="range"/>
        </div>
    }
}
