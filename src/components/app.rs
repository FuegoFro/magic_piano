use bit_set::BitSet;
use gloo::net::http::Request;
use itertools::Itertools;
use js_sys::{JsString, Uint8Array};
use leptos::{
    component, create_effect, create_local_resource, create_node_ref, create_signal,
    create_trigger, event_target_value, spawn_local, view, with, CollectView, IntoAttribute,
    IntoView, Signal, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate, SignalWith,
    SignalWithUntracked,
};
use web_sys::File;

use crate::components::keyboard_listener::KeyboardListener;
use crate::components::sheet_music::SheetMusic;
use crate::components::voice_control::{VoiceControl, VoiceState};
use crate::future_util::PromiseAsFuture;
use crate::opensheetmusicdisplay_bindings::OpenSheetMusicDisplay;
use crate::playback_manager::PlaybackManager;
use crate::song_data::SongData;

const SONGS: &[&str] = &[
    "A Million Stars",
    "Clouds On Fire",
    "Last Night Was The End Of The World",
    "Like Leave We'll Fall",
    "Lone Prairie",
    "Love Come Back To Me",
    "Mam'selle",
    "Though You're Gone (I Love You Still)",
];

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

#[derive(Clone, Eq, PartialEq)]
enum SongChoice {
    BuiltIn { name: String },
    Uploaded { file: File },
}

#[component]
pub fn App() -> impl IntoView {
    let (song_choice, set_song_choice) = create_signal(SongChoice::BuiltIn {
        name: SONGS[0].to_string(),
    });
    let on_reset_song = create_trigger();
    // Reset whenever the song name changes
    create_effect(move |_| song_choice.with(|_| on_reset_song.notify()));

    let file_input_ref = create_node_ref();

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

    let (start_cursor_index, set_start_cursor_index) = create_signal(0);
    let (current_cursor_index, set_current_cursor_index) = create_signal(0);

    let (osmd, set_osmd) = create_signal::<Option<OpenSheetMusicDisplay>>(None);
    let (song_data, set_song_data) = create_signal::<Option<SongData>>(None);
    let song_raw_data = create_local_resource(
        move || song_choice.get(),
        move |song_choice| async move {
            let data = match song_choice {
                SongChoice::BuiltIn { name } => {
                    Request::get(&format!("examples/{name}.mxl"))
                        .send()
                        .await
                        .unwrap()
                        // If we really cared about the extra copy of the data below we could use `.body()`
                        // here instead.
                        .binary()
                        .await
                        .unwrap()
                }
                SongChoice::Uploaded { file } => {
                    // It's probably a bit inefficient to read this data into Rust-land and then
                    // pass it back to JS-land, but oh well.
                    let blob: &web_sys::Blob = file.as_ref();
                    let array_buffer = blob.array_buffer().into_future().await.unwrap();
                    let typed_buff: Uint8Array = Uint8Array::new(&array_buffer);
                    let mut data = vec![0; typed_buff.length() as usize];
                    typed_buff.copy_to(&mut data);
                    data
                }
            };
            // The library takes in the binary data as a string (*shudders*) and JS uses arbitrary 16-bit
            // character codes (nominally utf-16, but doesn't need to be valid utf-16), so we need to make
            // these u16's and then shove it into a JsString. Interestingly that means just left-padding
            // them with 0's, not packing two u8's into a single u16 (which didn't work).
            let u16_data = data.iter().map(|d| *d as u16).collect_vec();
            JsString::from_char_code(&u16_data)
        },
    );
    // Load the song into osmd and extract the SongData.
    // Not using create_local_resource because we depend on osmd which doesn't impl Eq, which is
    // needed for Resource inputs.
    create_effect(move |_| {
        // Important that we track the usage of both of these before we enter `spawn_local`
        let load_promise = with!(|osmd, song_raw_data| {
            let Some(osmd) = osmd else { return None };
            let Some(song_raw_data) = song_raw_data else {
                return None;
            };
            Some(osmd.load(song_raw_data))
        });
        let Some(load_promise) = load_promise else {
            return;
        };
        spawn_local(async move {
            load_promise.into_future().await.unwrap();
            // We tracked this usage above
            osmd.with_untracked(|osmd| {
                // We shouldn't be able to get here without this set.
                let osmd = osmd.as_ref().unwrap();
                osmd.set_zoom(0.5);
                osmd.render();
                // Now load the song data
                let cursor = osmd.cursor().unwrap();
                cursor.reset();
                cursor.hide();
                let song_data = SongData::from_osmd(osmd);
                set_song_data.set(Some(song_data));
                // Reset and show the cursors
                for cursor in osmd.cursors() {
                    cursor.reset();
                    cursor.show();
                }
            });
        });
    });

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

    let is_loading = Signal::derive(move || {
        playback_manager.loading().get() || song_data.with(|song_data| song_data.is_none())
    });

    view! {
        <div class="flex flex-col items-start p-2 space-y-1 h-screen">
            <KeyboardListener
                playback_manager=playback_manager
                active_voices=active_voices
                set_start_cursor_index=set_start_cursor_index
                set_current_cursor_index=set_current_cursor_index
                on_reset_song=on_reset_song
            />
            <br/>
            <div class="flex flex-row items-baseline space-x-1">
                <p>"Pick a song:"</p>
                <select
                    class="border"
                    on:change=move |e| {
                        let new_value = event_target_value(&e);
                        set_song_choice
                            .set(SongChoice::BuiltIn {
                                name: new_value,
                            });
                    }
                >

                    // Apparently the API for initial selection is adding `selected`
                    // to the corresponding option >.>
                    // Also if this comment is inside the following block leptosfmt loses its mind
                    {SONGS
                        .iter()
                        .map(|&song_option| {
                            let attrs = if SONGS[0] == song_option {
                                vec![("selected", true.into_attribute())]
                            } else {
                                vec![]
                            };
                            view! {
                                <option {..attrs} value=song_option>
                                    {song_option}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
            </div>
            <div class="flex flex-row items-baseline space-x-1">
                <p>"Or upload a song: "</p>
                <input
                    _ref=file_input_ref
                    type="file"
                    accept=".xml,.mxl,.musicxml"
                    multiple=false
                    on:change=move |_| {
                        let input = file_input_ref.get().unwrap();
                        let Some(files) = input.files() else { return };
                        let Some(file) = files.get(0) else { return };
                        set_song_choice.set(SongChoice::Uploaded { file })
                    }
                />

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
            <div class="relative w-full h-full">
                // We always want this to be here so it can layout properly in the background,
                // but sometimes we overlay it with a loading div.
                <SheetMusic
                    song_data=song_data
                    start_cursor_index=start_cursor_index
                    current_cursor_index=current_cursor_index
                    osmd=osmd
                    set_osmd=set_osmd
                />

                {move || {
                    is_loading
                        .get()
                        .then(move || {
                            view! {
                                <div class="absolute top-0 left-0 bg-white flex items-center justify-center w-full h-full">
                                    <p class="w-fit">Loading...</p>
                                </div>
                            }
                        })
                }}

            </div>
        </div>
    }
}
