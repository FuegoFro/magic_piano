use std::collections::HashMap;

use bit_set::BitSet;
use leptos::{
    component, create_signal, ev, on_cleanup, window_event_listener, with, IntoView, Resource,
    Signal, SignalSet, SignalUpdate, WriteSignal,
};

use crate::playback_manager::PlaybackManager;
use crate::sampler::SamplerPlaybackGuard;

#[component]
pub fn KeyboardListener(
    #[prop(into)] playback_manager: Resource<(), PlaybackManager>,
    #[prop(into)] active_voices: Signal<BitSet>,
    set_index_to_show: WriteSignal<usize>,
) -> impl IntoView {
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
            let Some((cursor_index, newly_held_notes)) =
                playback_manager.start_notes_at_relative_index(song_index, active_voices)
            else {
                return;
            };
            set_index_to_show.set(cursor_index);

            set_held_notes.update(|held_notes| {
                held_notes.insert(key, newly_held_notes);
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

    // We don't actually render anything in the DOM, so don't return anything
}
