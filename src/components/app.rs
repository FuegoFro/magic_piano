use std::collections::HashMap;

use bit_set::BitSet;
use gloo::net::http::Request;
use itertools::Itertools;
use leptos::{
    component, create_effect, create_local_resource, create_signal, ev, event_target_value,
    on_cleanup, view, window_event_listener, with, CollectView, IntoView, Show, Signal, SignalGet,
    SignalGetUntracked, SignalSet, SignalUpdate, SignalWith, SignalWithUntracked,
};
use log::debug;

use crate::components::voice_control::{VoiceControl, VoiceState};
use crate::playback_manager::PlaybackManager;
use crate::sampler::SamplerPlaybackGuard;
use crate::song_data::SongData;

const SONGS: &[&str] = &["A Million Stars", "Lone Prairie", "Mam'selle"];

/// Converts a 0-100 volume to a gain multiplier by interpolating between the given min/max
/// relative decibel levels and then converting that to a multiplier. Min db should probably be
/// around -60 to -90.
/// Taken from https://www.reddit.com/r/programming/comments/9n2y0/stop_making_linear_volume_controls/
fn volume_to_gain(volume: u32, min_db: f32, max_db: f32) -> f32 {
    let factor = volume as f32 / 100.0;
    let db = min_db + (max_db - min_db) * factor;
    // The formula here is dB = 20*log10(Gn/G0)
    // Where:
    // - log10 is log base 10
    // - Gn is the new gain after changing by that much dB
    // - G0 is the original gain
    // If we set G0 = 1.0 as our initial reference gain, we can solve for Gn:
    // dB / 20 = log10(Gn)
    // 10 ^ (dB / 20) = Gn
    10.0f32.powf(db / 20.0)
}

#[component]
pub fn App() -> impl IntoView {
    let (song_name, set_song_name) = create_signal(SONGS[1].to_string());
    let (overall_volume, set_overall_volume) = create_signal(70u32);
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

    let active_voices = {
        let voice_mute_playbacks = voice_states
            .get_untracked()
            .into_iter()
            .map(|vs| vs.mute_playback_signal(any_voice_solo))
            .collect_vec();
        Signal::derive(move || {
            voice_mute_playbacks
                .iter()
                .enumerate()
                .filter(|(_, muted)| !muted.get())
                .map(|(voice, _)| voice)
                .collect::<BitSet>()
        })
    };

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

            let song_data = SongData::from_bytes(&data);
            for slice in song_data.slices.iter() {
                debug!("Slice: {:?}", slice.notes_by_voice);
            }
            song_data
        },
    );

    let playback_manager =
        create_local_resource(|| (), |_| async { PlaybackManager::initialize().await });
    // Mirror/translate the settings into the playback manager
    create_effect(move |_| {
        // Important! Don't blindly call update on the resource since that'll overwrite it during
        // init. Only update it if it already exists.
        if playback_manager.loading().get() {
            return;
        }
        playback_manager.update(|playback_manager| {
            let Some(playback_manager) = playback_manager else {
                return;
            };
            if let Some(song_data) = song_data.get() {
                playback_manager.set_song_data(song_data);
            }
        });
    });
    create_effect(move |_| {
        with!(|playback_manager, overall_volume| {
            if let Some(playback_manager) = playback_manager {
                playback_manager.set_overall_gain(volume_to_gain(*overall_volume, -30.0, 5.0));
            }
        })
    });
    for (voice, voice_state) in voice_states.get_untracked().into_iter().enumerate() {
        let voice_volume = voice_state.volume;
        create_effect(move |_| {
            with!(|playback_manager, voice_volume| {
                if let Some(playback_manager) = playback_manager {
                    playback_manager
                        .set_voice_gain(voice, volume_to_gain(*voice_volume, -30.0, 5.0));
                }
            })
        });
    }

    let (_, set_held_notes) =
        create_signal::<HashMap<String, Vec<SamplerPlaybackGuard>>>(HashMap::new());

    let keydown_handle = window_event_listener(ev::keydown, move |event| {
        let has_modifier =
            event.meta_key() || event.ctrl_key() || event.shift_key() || event.alt_key();
        if has_modifier {
            return;
        }

        let key = event.key();
        // let song_index = "qwerasdfzxcvuiopjkl;m,./".find(event.key().as_str())?;
        let Some(song_index) = "qwertyuiopasdfghjkl;zxcvbnm,./".find(key.as_str()) else {
            return;
        };

        event.prevent_default();

        if event.repeat() {
            // Only send events on the initial keypress.
            // This needs to happen after we've prevented default.
            return;
        }
        with!(|playback_manager, active_voices| {
            let Some(playback_manager) = playback_manager else {
                return;
            };
            set_held_notes.update(|held_notes| {
                held_notes.insert(
                    key,
                    playback_manager.start_notes_at_relative_index(song_index, active_voices),
                );
            });
        });
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
                        view! { <VoiceControl voice_state=vs any_voice_solo=any_voice_solo/> }
                    })
                    .collect_view()}
            </div>
            <div class="flex flex-row space-x-1">
                <p>"Overall Volume:"</p>
                <input
                    type="range"
                    max=100
                    prop:value=overall_volume
                    on:input=move |e| {
                        set_overall_volume.set(event_target_value(&e).parse().unwrap());
                    }
                />
            </div>
            <Show
                when=move || !playback_manager.loading().get() && !song_data.loading().get()
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
