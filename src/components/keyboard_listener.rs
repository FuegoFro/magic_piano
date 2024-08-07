use std::collections::HashMap;

use bit_set::BitSet;
use codee::string::FromToStringCodec;
use leptos::{
    component, create_effect, create_node_ref, create_signal, ev, on_cleanup, view,
    window_event_listener, with, IntoAttribute, IntoView, Resource, RwSignal, Signal, SignalGet,
    SignalGetUntracked, SignalSet, SignalUpdate, Trigger, WriteSignal,
};
use leptos_use::storage::use_local_storage;

use crate::playback_manager::PlaybackManager;
use crate::sampler::SamplerPlaybackGuard;

pub const LETTERS: &str = "qwerasdfzxcvuiopjkl;m,./";
// pub const LETTERS: &str = "qwertyuiopasdfghjkl;zxcvbnm,./";

#[component]
pub fn KeyboardListener(
    playback_manager: Resource<(), PlaybackManager>,
    #[prop(into)] active_voices: Signal<BitSet>,
    set_start_cursor_index: WriteSignal<usize>,
    set_current_cursor_index: WriteSignal<usize>,
    // Lets us know when to reset things.
    #[prop(into)] on_reset_song: Trigger,
) -> impl IntoView {
    let (_, set_held_notes) =
        create_signal::<HashMap<String, Vec<SamplerPlaybackGuard>>>(HashMap::new());
    let start_song_index = RwSignal::new(0);
    let most_recent_song_index = RwSignal::new(0);
    // Reset the indices when we have a new song.
    create_effect(move |_| {
        on_reset_song.track(); // This will re-trigger the effect.
        start_song_index.set(0);
        most_recent_song_index.set(0);
        set_start_cursor_index.set(0);
        set_current_cursor_index.set(0);
    });

    // Sync start index to start cursor
    create_effect(move |_| {
        with!(|playback_manager| {
            let Some(playback_manager) = playback_manager else {
                return;
            };
            if let Some(max_song_index) = playback_manager.max_song_index() {
                if max_song_index < start_song_index.get() {
                    start_song_index.set(max_song_index);
                }
            }
            if let Some(cursor_index) =
                playback_manager.cursor_index_for_song_index(start_song_index.get())
            {
                set_start_cursor_index.set(cursor_index);
            };
        });
    });

    let keydown_handle = window_event_listener(ev::keydown, move |event| {
        let has_modifier =
            event.meta_key() || event.ctrl_key() || event.shift_key() || event.alt_key();
        if has_modifier {
            return;
        }

        // First check if this is a key press that we want to do something with
        let Some(action) = get_no_modifiers_key_action(
            event.key(),
            playback_manager,
            active_voices,
            set_held_notes,
            start_song_index,
            most_recent_song_index,
            set_current_cursor_index,
        ) else {
            return;
        };

        // If so, prevent default so we don't do things like scroll the page with space
        event.prevent_default();

        // Then ignore repeats. Note this needs to happen after we've determined it's something we
        // care about and prevented default.
        if event.repeat() && !action.allow_repeats {
            return;
        }

        // Finally take the action.
        (action.action)();
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

    let details_node_ref = create_node_ref();
    let (has_seen_controls, set_has_seen_controls, _) =
        use_local_storage::<bool, FromToStringCodec>("has_seen_controls");
    // Annoyingly the mere presence of this attribute determines whether it's open
    let attrs = if has_seen_controls.get_untracked() {
        vec![]
    } else {
        vec![("open", true.into_attribute())]
    };

    view! {
        <details
            {..attrs}
            class="outline outline-2 outline-slate-500 bg-slate-100 p-1 rounded"
            _ref=details_node_ref
            on:toggle=move |_| {
                if !details_node_ref.get().map(|d| d.open()).unwrap_or(true) {
                    set_has_seen_controls.set(true);
                }
            }
        >

            <summary>
                <h1 class="text-xl inline">Controls</h1>
            </summary>
            <div class="flex flex-row items-baseline space-x-1">
                <div class="size-3 h-4 w-4 rounded-full bg-noteCursor"></div>
                <h2 class="text-lg font-medium">Playing notes</h2>
            </div>
            <p>
                "Each key on your keyboard will play the notes at a position in a song. "
                <code class="bg-slate-200">Q</code> " is the first position, "
                <code class="bg-slate-200">W</code> ", the second, "
                <code class="bg-slate-200">E</code> ", the third, and so on."
            </p>
            <p>"The keys are designed to be played with one hand:"</p>
            <p>
                <code class="bg-slate-200">"Q W E R"</code>
            </p>
            <p>
                <code class="bg-slate-200">"A S D F"</code>
            </p>
            <p>
                <code class="bg-slate-200">"Z X C V"</code>
            </p>
            <p>"And then the same on the right hand:"</p>
            <p>
                <code class="bg-slate-200">"U I O P"</code>
            </p>
            <p>
                <code class="bg-slate-200">"J K L ;"</code>
            </p>
            <p>
                <code class="bg-slate-200">"M , . /"</code>
            </p>
            <div class="flex flex-row items-baseline space-x-1">
                <div class="size-3 h-4 w-4 rounded-full bg-startCursor"></div>
                <h2 class="text-lg font-medium">Changing starting position</h2>
            </div>

            <p>
                "The positions are relative to the current start (indicated in teal). This can be "
                "moved to just after the most recently played note by pressing the space bar, or "
                "can be adjusted with the left/right arrows. It can be reset with backtick ("
                <code class="bg-slate-200">"`"</code> ")."
            </p>
            <img class="my-1" src="examples/keyboard.png"/>
            <p>"(you can collapse these instructions by clicking on \"Controls\" above)"</p>
        </details>
    }
}

struct KeyAction {
    action: Box<dyn FnOnce()>,
    allow_repeats: bool,
}

impl KeyAction {
    fn new(action: impl FnOnce() + 'static) -> Self {
        Self {
            action: Box::new(action),
            allow_repeats: false,
        }
    }
    fn new_with_repeats(action: impl FnOnce() + 'static) -> Self {
        Self {
            action: Box::new(action),
            allow_repeats: true,
        }
    }
}

/// Returns whether this key was used/valid.
fn get_no_modifiers_key_action(
    key: String,
    playback_manager: Resource<(), PlaybackManager>,
    active_voices: Signal<BitSet>,
    set_held_notes: WriteSignal<HashMap<String, Vec<SamplerPlaybackGuard>>>,
    start_song_index: RwSignal<usize>,
    most_recent_song_index: RwSignal<usize>,
    set_current_cursor_index: WriteSignal<usize>,
) -> Option<KeyAction> {
    let action = if key == " " {
        KeyAction::new(move || {
            let new_start_song_index = most_recent_song_index.get() + 1;
            start_song_index.set(new_start_song_index);
        })
    } else if key == "ArrowLeft" || key == "ArrowRight" {
        KeyAction::new_with_repeats(move || {
            let new_start_song_index = if key == "ArrowLeft" {
                start_song_index.get().saturating_sub(1)
            } else {
                start_song_index.get() + 1
            };
            start_song_index.set(new_start_song_index);
        })
    } else if key == "`" {
        KeyAction::new(move || {
            start_song_index.set(0);
        })
    } else if let Some(offset) = LETTERS.find(key.as_str()) {
        KeyAction::new(move || {
            with!(|playback_manager, active_voices, start_song_index| {
                let song_index = start_song_index + offset;
                most_recent_song_index.set(song_index);
                let Some(playback_manager) = playback_manager else {
                    return;
                };
                let Some((cursor_index, newly_held_notes)) =
                    playback_manager.start_notes_at_relative_index(song_index, active_voices)
                else {
                    return;
                };
                set_current_cursor_index.set(cursor_index);

                set_held_notes.update(|held_notes| {
                    held_notes.insert(key, newly_held_notes);
                });
            });
        })
    } else {
        return None;
    };

    Some(action)
}
