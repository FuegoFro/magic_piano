use std::collections::HashMap;

use bit_set::BitSet;
use leptos::prelude::*;

use crate::playback_manager::PlaybackManager;
use crate::sampler::SamplerPlaybackGuard;

#[component]
pub fn MobileControls(
    playback_manager: LocalResource<RwSignal<PlaybackManager, LocalStorage>>,
    #[prop(into)] active_voices: Signal<BitSet>,
    start_song_index: RwSignal<usize>,
    most_recent_song_index: RwSignal<usize>,
    set_current_cursor_index: WriteSignal<usize>,
) -> impl IntoView {
    let (playing_notes, set_playing_notes) = signal_local::<HashMap<String, Vec<SamplerPlaybackGuard>>>(HashMap::new());
    let (has_moved_next, set_has_moved_next) = signal_local(false);

    let handle_reset = move |_| {
        let start_index = start_song_index.get();
        most_recent_song_index.set(start_index);
        // Update cursor to match the start position
        let playback_manager = playback_manager.read();
        if let Some(playback_manager) = &*playback_manager {
            if let Some(cursor_index) = playback_manager
                .read()
                .cursor_index_for_song_index(start_index)
            {
                set_current_cursor_index.set(cursor_index);
            }
        }
        set_has_moved_next.set(false);
    };

    let handle_set_start = move |_| {
        let current_song_index = most_recent_song_index.get();
        start_song_index.set(current_song_index);
    };

    let play_note_at_index = move |song_index: usize, button_id: String| {
        let playback_manager = playback_manager.read();
        let active_voices = active_voices.read();
        let Some(playback_manager) = &*playback_manager else {
            return;
        };
        let Some((cursor_index, newly_held_notes)) = playback_manager
            .write()
            .start_notes_at_relative_index(song_index, &*active_voices)
        else {
            return;
        };
        set_current_cursor_index.set(cursor_index);
        set_playing_notes.update(|held_notes| {
            held_notes.insert(button_id, newly_held_notes);
        });
    };

    let stop_playing = move |button_id: String| {
        set_playing_notes.update(|notes| {
            notes.remove(&button_id);
        });
    };

    let handle_previous_press = move |_| {
        let current = most_recent_song_index.get();
        let new_index = current.saturating_sub(1);
        play_note_at_index(new_index, "previous".to_string());
        most_recent_song_index.set(new_index);
    };

    let handle_previous_release = move |_| {
        stop_playing("previous".to_string());
    };

    let handle_next_press = move |_| {
        let current = most_recent_song_index.get();
        if has_moved_next.get() {
            // If we've moved next before, increment normally
            let new_index = current + 1;
            play_note_at_index(new_index, "next".to_string());
            most_recent_song_index.set(new_index);
        } else {
            // First time pressing next, play the current position (0)
            set_has_moved_next.set(true);
            play_note_at_index(current, "next".to_string());
            // Don't increment most_recent_song_index on first press
        }
    };

    let handle_next_release = move |_| {
        stop_playing("next".to_string());
    };

    view! {
        <div class="mobile-controls fixed bottom-0 left-0 right-0 bg-gray-800 p-4 justify-around items-center">
            <button
                class="p-3 bg-gray-700 rounded-full hover:bg-gray-600 active:bg-gray-500 transition-colors"
                on:click=handle_reset
                aria-label="Reset to start"
            >
                <svg class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z"/>
                </svg>
            </button>

            <button
                class="p-3 bg-gray-700 rounded-full hover:bg-gray-600 active:bg-gray-500 transition-colors"
                on:pointerdown=handle_previous_press
                on:pointerup=handle_previous_release
                on:pointercancel=handle_previous_release
                on:pointerout=handle_previous_release
                aria-label="Previous note"
            >
                <svg class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
                </svg>
            </button>

            <button
                class="p-3 bg-gray-700 rounded-full hover:bg-gray-600 active:bg-gray-500 transition-colors"
                on:pointerdown=handle_next_press
                on:pointerup=handle_next_release
                on:pointercancel=handle_next_release
                on:pointerout=handle_next_release
                aria-label="Next note"
            >
                <svg class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
                </svg>
            </button>

            <button
                class="p-3 bg-gray-700 rounded-full hover:bg-gray-600 active:bg-gray-500 transition-colors"
                on:click=handle_set_start
                aria-label="Set start position"
            >
                <svg class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M12 2L8 6h3v10h2V6h3L12 2z"/>
                </svg>
            </button>
        </div>
    }
}