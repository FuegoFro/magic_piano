use bit_set::BitSet;
use gloo::net::http::Request;
use itertools::Itertools;
use js_sys::{JsString, Uint8Array};
use leptos::prelude::*;
use web_sys::File;

use crate::components::keyboard_listener::KeyboardListener;
use crate::components::mobile_controls::MobileControls;
use crate::components::sheet_music::SheetMusic;
use crate::components::voice_control::{VoiceControl, VoiceState};
use crate::future_util::PromiseAsFuture;
use crate::playback_manager::PlaybackManager;
use crate::song_data::SongData;

const SONGS: &[&str] = &[
    "A Million Stars",
    "Alone with the Crowd",
    "Cheer Up, Charlie",
    "Close Your Eyes In Sleep",
    "Clouds On Fire",
    "Crazy Blackbird Tag",
    "Dear Old Pal of Mine",
    "Dinah",
    "Ebb Tide",
    "I Have Dreamed",
    "Ireland, My Ireland",
    "Johanna",
    "La Vie En Rose",
    "Last Night Was The End Of The World",
    "Like Leave We'll Fall",
    "Lone Prairie",
    "Lonely For You Am I",
    "Love Come Back To Me",
    "Mam'selle",
    "New York Ain't New York Anymore",
    "Ring The Bells of Notre Dame",
    "She Stole My Heart Away",
    "Silvery Moonlight",
    "Sing an Old Time Song Again",
    "Smile",
    "Superman",
    "That's Why I Get the Blues When It Rains",
    "The Shadow of Your Smile",
    "This Is All I Ask (Stars in the Sky)",
    "Though You're Gone (I Love You Still)",
    "To My Beautiful Lifelong Friends",
    "What Miracle Has Made You The Way You Are",
    "When You And I Were Young, Maggie",
    "Without a Song",
    "Yesterday I Heard the Rain",
    "You Are The One",
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
pub enum SongChoice {
    BuiltIn { name: String },
    Uploaded { file: File },
}

fn create_voice_states(num_voices: usize) -> Vec<VoiceState> {
    let names = if num_voices == 4 {
        ["Tenor", "Lead", "Bari", "Bass"]
            .into_iter()
            .map(|name| name.to_string())
            .collect_vec()
    } else {
        (1..=num_voices)
            .map(|num| format!("Voice {num}"))
            .collect_vec()
    };

    names
        .into_iter()
        .map(|name| VoiceState::new(name.to_string()))
        .collect_vec()
}

#[component]
pub fn App() -> impl IntoView {
    let (song_choice, set_song_choice) = signal_local(SongChoice::BuiltIn {
        name: SONGS[0].to_string(),
    });
    let on_reset_song = Trigger::new();
    // Reset whenever the song name changes
    Effect::new(move |_| song_choice.with(|_| on_reset_song.notify()));

    let file_input_ref = NodeRef::new();

    let (start_cursor_index, set_start_cursor_index) = signal(0);
    let (current_cursor_index, set_current_cursor_index) = signal(0);
    
    let start_song_index = RwSignal::new(0);
    let most_recent_song_index = RwSignal::new(0);

    let (song_data, set_song_data) = signal::<Option<SongData>>(None);
    let song_raw_data = LocalResource::new(move || async move {
        let song_choice = song_choice.get();
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
    });

    let (overall_volume, set_overall_volume) = signal(70u32);
    let num_voices = Memo::new(move |_| {
        song_data.with(|song_data| {
            song_data
                .as_ref()
                .map(|sd| sd.voice_index_mapping.len())
                .unwrap_or(4)
        })
    });
    let voice_states = Memo::new_owning(move |previous_voice_states: Option<Vec<VoiceState>>| {
        let num_voices = num_voices.get();
        if let Some(previous_voice_states) = previous_voice_states {
            if previous_voice_states.len() == num_voices {
                return (previous_voice_states, false);
            }
        }
        (create_voice_states(num_voices), true)
    });
    let any_voice_solo =
        Signal::derive(move || voice_states.with(|vss| vss.iter().any(|vs| vs.solo.get())));

    let active_voices = Signal::derive(move || {
        voice_states
            .get()
            .into_iter()
            .map(|vs| vs.mute_playback_signal(any_voice_solo))
            .enumerate()
            .filter(|(_, muted)| !muted.get())
            .map(|(voice, _)| voice)
            .collect::<BitSet>()
    });

    let playback_manager: LocalResource<RwSignal<PlaybackManager, LocalStorage>> =
        LocalResource::new(|| async { RwSignal::new_local(PlaybackManager::initialize().await) });
    // Mirror/translate the settings into the playback manager
    Effect::new(move |_| {
        if let (Some(playback_manager), Some(song_data)) =
            (&*playback_manager.read(), &*song_data.read())
        {
            playback_manager.write().set_song_data(song_data.clone());
        }
    });
    Effect::new(move |_| {
        if let Some(playback_manager) = &*playback_manager.read() {
            playback_manager.write().set_overall_gain(volume_to_gain(
                *overall_volume.read(),
                -30.0,
                5.0,
            ));
        }
    });

    Effect::new(move |_| {
        for (voice, voice_state) in voice_states.get().into_iter().enumerate() {
            if let Some(playback_manager) = &*playback_manager.read() {
                playback_manager.write().set_voice_gain(
                    voice,
                    volume_to_gain(*voice_state.volume.read(), -30.0, 5.0),
                );
            }
        }
    });

    let is_loading = Signal::derive(move || {
        playback_manager.with(|pm| pm.is_none()) || song_data.with(|song_data| song_data.is_none())
    });

    view! {
        <div class="flex flex-col items-start p-2 space-y-1 h-screen">
            <KeyboardListener
                playback_manager=playback_manager
                active_voices=active_voices
                start_song_index=start_song_index
                most_recent_song_index=most_recent_song_index
                set_start_cursor_index=set_start_cursor_index
                set_current_cursor_index=set_current_cursor_index
                on_reset_song=on_reset_song
            />
            <br />
            <div class="flex flex-row items-baseline space-x-1">
                <p>"Pick a song:"</p>
                <select
                    class="border"
                    on:change:target=move |ev| {
                        let new_value = ev.target().value();
                        set_song_choice
                            .set(SongChoice::BuiltIn {
                                name: new_value,
                            });
                    }
                >

                    {SONGS
                        .iter()
                        .map(|&song_option| {
                            view! {
                                <option selected={SONGS[0] == song_option} value=song_option>
                                    {song_option}
                                </option>
                            }
                        })
                        .collect_vec()}
                </select>
            </div>
            <div class="flex flex-row items-baseline space-x-1">
                <p>"Or upload a song: "</p>
                <input
                    node_ref=file_input_ref
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
                {move || {
                    voice_states
                        .get()
                        .into_iter()
                        .map(|vs| {
                            view! { <VoiceControl voice_state=vs any_voice_solo=any_voice_solo /> }
                        })
                        .collect_vec()
                }}

            </div>
            <div class="flex flex-row space-x-1">
                <p>"Overall Volume:"</p>
                <input
                    type="range"
                    max=100
                    prop:value=overall_volume
                    on:input:target=move |ev| {
                        set_overall_volume.set(ev.target().value().parse().unwrap());
                    }
                />

            </div>
            <div class="relative w-full h-full">
                // We always want this to be here so it can layout properly in the background,
                // but sometimes we overlay it with a loading div.
                <SheetMusic
                    active_voices=active_voices
                    song_data=song_data
                    start_cursor_index=start_cursor_index
                    current_cursor_index=current_cursor_index
                    song_raw_data=song_raw_data
                    set_song_data=set_song_data
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
            <MobileControls
                playback_manager=playback_manager
                active_voices=active_voices
                start_song_index=start_song_index
                most_recent_song_index=most_recent_song_index
                set_current_cursor_index=set_current_cursor_index
            />
        </div>
    }
}
