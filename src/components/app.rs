use gloo::net::http::Request;
use itertools::Itertools;
use leptos::{
    component, create_local_resource, create_signal, ev, event_target_value, on_cleanup, view,
    window_event_listener, with, CollectView, IntoView, RwSignal, Show, Signal, SignalGet,
    SignalGetUntracked, SignalSet, SignalUpdate, SignalWith,
};
use midly::Smf;
use std::collections::HashSet;
use web_sys::AudioContext;

use crate::components::voice_control::VoiceControl;
use crate::sampler::Sampler;
use crate::song::Song;
use crate::{event_to_song_index, start_song_index, stop_song_index, NOTES};

const SONGS: [&str; 2] = ["A Million Stars", "Mam'selle"];

#[derive(Clone)]
struct VoiceState {
    name: String,
    mute: RwSignal<bool>,
    solo: RwSignal<bool>,
}

impl VoiceState {
    fn new(name: String) -> Self {
        Self {
            name,
            mute: RwSignal::new(false),
            solo: RwSignal::new(false),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (song_name, set_song_name) = create_signal(SONGS[0].to_string());
    let sampler = create_local_resource(
        || (),
        |_| async move { Sampler::initialize(AudioContext::new().unwrap(), &NOTES).await },
    );

    let song_data = create_local_resource(
        move || song_name.get(),
        |song_name| async move {
            let data = Request::get(&format!("examples/{song_name}.mid"))
                .send()
                .await
                .unwrap()
                .binary()
                .await
                .unwrap();
            let midi_file =
                Smf::parse(&data).unwrap_or_else(|e| std::panic!("Unable to parse midi file: {e}"));
            Song::from_smf(&midi_file)
        },
    );

    // We don't want to overwrite the voice_states directly, but we're wrapping it in a signal so
    // it's trivially copyable.
    let (voice_states, _) = create_signal(
        ["Tenor", "Lead", "Bari", "Bass"]
            .into_iter()
            .map(|name| VoiceState::new(name.to_string()))
            .collect_vec(),
    );
    let any_voice_solo =
        Signal::derive(move || voice_states.with(|vss| vss.iter().any(|vs| vs.solo.get())));
    // TODO - remove this and just use voice states directly
    let voices_hash = Signal::derive(move || {
        with!(|voice_states, any_voice_solo| {
            voice_states
                .iter()
                .enumerate()
                // The voice is "on" if there are no other soloists OR it is a solo, and if it's not muted.
                .filter(|(_, vs)| (!any_voice_solo || vs.solo.get()) && !vs.mute.get())
                .map(|(idx, _)| idx)
                .collect::<HashSet<_>>()
        })
    });

    let keydown_handle = window_event_listener(ev::keydown, move |e| {
        if let Some(song_index) = event_to_song_index(&e) {
            with!(|song_data, voices_hash| {
                let Some(song_data) = song_data else { return };
                sampler.update(|sampler| {
                    let Some(sampler) = sampler else { return };
                    start_song_index(sampler, &song_data, voices_hash, song_index);
                })
            });
        }
    });
    on_cleanup(move || keydown_handle.remove());

    let keyup_handle = window_event_listener(ev::keyup, move |e| {
        if let Some(song_index) = event_to_song_index(&e) {
            with!(|song_data, voices_hash| {
                let Some(song_data) = song_data else { return };
                sampler.update(|sampler| {
                    let Some(sampler) = sampler else { return };
                    stop_song_index(sampler, &song_data, voices_hash, song_index);
                })
            });
        }
    });
    on_cleanup(move || keyup_handle.remove());

    view! {
        <div class="flex flex-col items-start p-2 space-y-1 h-screen">
            <div class="flex flex-row items-baseline space-x-1">
                <p>"Pick a song:"</p>
                <select
                    class="border"
                    on:change=move |e| {
                        let new_value = event_target_value(&e);
                        set_song_name.set(new_value);
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
                    .map(|vs| {
                        view! {
                            <VoiceControl
                                name=vs.name
                                mute=vs.mute
                                solo=vs.solo
                                any_solo=any_voice_solo
                                on_toggle_mute=move |()| {
                                    vs.mute
                                        .update(move |m| {
                                            *m = !*m;
                                        })
                                }

                                on_toggle_solo=move |()| vs.solo.update(move |s| *s = !*s)
                            />
                        }
                    })
                    .collect_view()}
            </div>
            <Show
                when=move || !sampler.loading().get() && !song_data.loading().get()
                fallback=|| {
                    view! {
                        <div class="flex items-center justify-center w-full h-full">
                            <p class="w-fit">Loading...</p>
                        </div>
                    }
                }
            >
                <img src=move || format!("examples/{}.png", song_name.get())/>
            </Show>
        </div>
    }
}
