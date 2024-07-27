use bit_set::BitSet;
use gloo::net::http::Request;
use itertools::Itertools;
use js_sys::JsString;
use leptos::{
    component, create_effect, create_local_resource, create_signal, event_target_value,
    spawn_local, view, with, CollectView, IntoView, Signal, SignalGet, SignalGetUntracked,
    SignalSet, SignalUpdate, SignalWith, SignalWithUntracked,
};

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

#[component]
pub fn App() -> impl IntoView {
    let (song_name, set_song_name) = create_signal(SONGS[0].to_string());
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

    let (index_to_show, set_index_to_show) = create_signal(0);

    let (osmd, set_osmd) = create_signal::<Option<OpenSheetMusicDisplay>>(None);
    let (song_data, set_song_data) = create_signal::<Option<SongData>>(None);
    let song_raw_data = create_local_resource(
        move || song_name.get(),
        move |song_name| async move {
            let data = Request::get(&format!("examples/{song_name}.mxl"))
                .send()
                .await
                .unwrap()
                // If we really cared about the extra copy of the data below we could use `.body()`
                // here instead.
                .binary()
                .await
                .unwrap();
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
                set_index_to_show.set(0);
                osmd.set_zoom(0.5);
                osmd.render();
                // Now load the song data
                let cursor = osmd.cursor().unwrap();
                cursor.reset();
                cursor.hide();
                let song_data = SongData::from_osmd(osmd);
                set_song_data.set(Some(song_data));
                cursor.reset();
                cursor.show();
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
        <KeyboardListener
            playback_manager=playback_manager
            active_voices=active_voices
            set_index_to_show=set_index_to_show
        />
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
            <div class="relative w-full h-full">
                // We always want this to be here so it can layout properly in the background,
                // but sometimes we overlay it with a loading div.
                <SheetMusic index_to_show=index_to_show osmd=osmd set_osmd=set_osmd/>

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
