use std::collections::{HashMap, HashSet};

use gloo::net::http::Request;
use itertools::Itertools;
use leptos::{
    component, create_local_resource, create_signal, ev, event_target_value, on_cleanup, view,
    window_event_listener, with, CollectView, IntoView, RwSignal, Show, Signal, SignalGet,
    SignalGetUntracked, SignalSet, SignalUpdate, SignalWith, SignalWithUntracked,
};
use log::debug;
use midly::Smf;
use web_sys::AudioContext;

use crate::components::voice_control::VoiceControl;
use crate::sampler::{Sampler, SamplerPlaybackGuard};
use crate::song::Song;
use crate::{event_to_song_index, start_song_index, NOTES};

const SONGS: &[&str] = &["A Million Stars", "Lone Prairie", "Mam'selle"];

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
    let (song_name, set_song_name) = create_signal(SONGS[1].to_string());
    let (overall_volume, set_overall_volume) = create_signal(50u32);
    let overall_gain = Signal::derive(move || overall_volume.get() as f32 / 100.0);
    let sampler = create_local_resource(
        || (),
        move |_| async move {
            let sampler = Sampler::initialize(AudioContext::new().unwrap(), &NOTES).await;
            sampler.set_overall_gain(overall_gain.get_untracked());
            sampler
        },
    );
    let (_, set_held_notes) =
        create_signal::<HashMap<String, Vec<SamplerPlaybackGuard>>>(HashMap::new());

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
            let song_data = Song::from_smf(&midi_file);
            for slice in song_data.slices.iter() {
                debug!("Slice: {:?}", slice.notes_by_voice);
            }
            song_data
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

    let keydown_handle = window_event_listener(ev::keydown, move |event| {
        let key = event.key();
        if let Some(song_index) = event_to_song_index(event) {
            with!(|song_data, voices_hash| {
                let Some(song_data) = song_data else { return };
                sampler.update(|sampler| {
                    let Some(sampler) = sampler else { return };
                    set_held_notes.update(|held_notes| {
                        held_notes.insert(
                            key,
                            start_song_index(sampler, song_data, voices_hash, song_index),
                        );
                    });
                })
            });
        }
    });
    on_cleanup(move || keydown_handle.remove());

    let keyup_handle = window_event_listener(ev::keyup, move |event| {
        let has_modifier =
            event.meta_key() || event.ctrl_key() || event.shift_key() || event.alt_key();
        if has_modifier {
            return;
        }

        set_held_notes.update(|held_notes| {
            held_notes.remove(&event.key());
        });
    });
    on_cleanup(move || keyup_handle.remove());

    view! {
        <div class="flex flex-col items-start p-2 space-y-1 h-screen">
            <h1 class="text-xl">Controls</h1>
            <p>
                "Each key on your keyboard represents an absolute position in a song. "
                <code class="bg-slate-200">Q</code> " is the first position, "
                <code class="bg-slate-200">W</code> ", the second, and so on."
            </p>
            <p>
                "The keys are " <code class="bg-slate-200">Q</code> " through "
                <code class="bg-slate-200">P</code> ", then " <code class="bg-slate-200">A</code>
                " through " <code class="bg-slate-200">;</code> ", and then "
                <code class="bg-slate-200">Z</code> " through " <code class="bg-slate-200">/</code>
                .
            </p>
            <br/>
            <div class="flex flex-row items-baseline space-x-1">
                <p>"Pick a song:"</p>
                <select
                    class="border"
                    on:change=move |e| {
                        let new_value = event_target_value(&e);
                        set_song_name.set(new_value);
                    }
                >

                    // Apparently the API for initial selection is adding `selected`
                    // to the corresponding option >.>
                    // Also if this comment is inside the following block leptosfmt loses its mind
                    {SONGS
                        .iter()
                        .map(|&song_option| {
                            if song_name.with_untracked(|sn| sn == song_option) {
                                view! {
                                    <option selected value=song_option>
                                        {song_option}
                                    </option>
                                }
                            } else {
                                view! { <option value=song_option>{song_option}</option> }
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
            <div class="flex flex-row space-x-1">
                <p>"Overall Volume:"</p>
                <input
                    type="range"
                    max=100
                    prop:value={overall_volume}
                    on:input=move |e| {
                        set_overall_volume.set(event_target_value(&e).parse().unwrap());
                        sampler.with(|sampler| sampler.as_ref().map(|sampler| sampler.set_overall_gain(overall_gain.get())));
                    }/>
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
