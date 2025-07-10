use leptos::prelude::*;

#[derive(Clone)]
pub struct VoiceState {
    pub name: String,
    pub mute: RwSignal<bool>,
    pub solo: RwSignal<bool>,
    pub volume: RwSignal<u32>,
}

impl VoiceState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            mute: RwSignal::new(false),
            solo: RwSignal::new(false),
            volume: RwSignal::new(70),
        }
    }

    pub fn mute_playback_signal(&self, any_voice_solo: Signal<bool>) -> Signal<bool> {
        let mute = self.mute;
        let solo = self.solo;
        Signal::derive(move || mute.get() || (!solo.get() && any_voice_solo.get()))
    }
}

#[component]
pub fn VoiceControl(
    voice_state: VoiceState,
    #[prop(into)] any_voice_solo: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center p-4 border border-black border-solid rounded-sm">
            <p class:text-red-600={voice_state.mute_playback_signal(any_voice_solo)}>{voice_state.name.clone()}</p>
            <div class="flex flex-row space-x-1">
                <button
                    class="border border-black rounded-sm px-1"
                    class:bg-red-600=voice_state.mute on:click=move |_| toggle_and_disable(voice_state.mute, voice_state.solo)>
                    Mute
                </button>
                <button
                    class="border border-black rounded-sm px-1"
                    class:bg-sky-600=voice_state.solo on:click=move |_| toggle_and_disable(voice_state.solo, voice_state.mute)>
                    Solo
                </button>
            </div>
            <p>"Volume"</p>
            <input type="range"
                max="100"
                prop:value={voice_state.volume}
                // TODO RIGHTNOW - Verify this works
                on:input:target=move |ev| {
                    voice_state.volume.set(ev.target().value().parse().unwrap());
                }
                />
        </div>
    }
}

fn toggle_and_disable(to_toggle: RwSignal<bool>, to_disable_if_other_active: RwSignal<bool>) {
    to_toggle.update(|t| *t = !*t);
    if to_toggle.get() {
        to_disable_if_other_active.set(false);
    }
}
