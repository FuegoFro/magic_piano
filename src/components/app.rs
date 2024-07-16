use crate::components::voice_control::VoiceControl;
use leptos::{
    component, create_signal, event_target_value, view, CollectView, IntoView, RwSignal, Signal,
    SignalGet, SignalGetUntracked, SignalSet, SignalUpdate, SignalWith,
};

const SONGS: [&str; 2] = ["A Million Stars", "Mam'selle"];

#[component]
pub fn App() -> impl IntoView {
    let (song, set_song) = create_signal(SONGS[0].to_string());
    // We don't want to overwrite the voice_states directly, but we're wrapping it in a signal so
    // it's trivially copyable.
    let (voice_states, _) = create_signal(
        ["Tenor", "Lead", "Bari", "Bass"]
            .into_iter()
            .map(|name| (name.to_string(), RwSignal::new(false), RwSignal::new(false)))
            .collect::<Vec<_>>(),
    );
    let any_voice_solo =
        Signal::derive(move || voice_states.with(|vs| vs.iter().any(|(_, _, solo)| solo.get())));

    view! {
        <div class="flex flex-col items-start p-2">
            <div class="flex flex-row items-baseline">
                <p>Pick a song:</p>
                <select
                    class="border"
                    on:change=move |e| {
                        let new_value = event_target_value(&e);
                        set_song.set(new_value);
                    }
                >

                    {SONGS
                        .iter()
                        .map(|song_option| {
                            view! {
                                // Need this lambda with to_string because otherwise the *stylers* macro parser chokes (?!?!?!)
                                <option value=move || {
                                    song_option.to_string()
                                }>{*song_option}</option>
                            }
                        })
                        .collect_view()}
                </select>
            </div>
            <div class="flex flex-row space-x-1">
                {voice_states
                    .get_untracked()
                    .into_iter()
                    .map(|(name, mute, solo)| {
                        view! {
                            <VoiceControl
                                name=name
                                mute=mute
                                solo=solo
                                any_solo=any_voice_solo
                                on_toggle_mute=move |()| {
                                    mute.update(move |m| {
                                        *m = !*m;
                                    })
                                }

                                on_toggle_solo=move |()| solo.update(move |s| *s = !*s)
                            />
                        }
                    })
                    .collect_view()}
            </div>
            <img src=move || format!("examples/{}.png", song.get())/>
        </div>
    }
}
