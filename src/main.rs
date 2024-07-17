use std::collections::HashSet;
use std::panic;

use leptos::{mount_to_body, view};
use web_sys::KeyboardEvent;

use song::Song;

use crate::components::app::App;
use crate::sampler::{Sampler, SamplerPlaybackGuard};

mod components;
mod sampler;
mod song;

fn main() {
    console_log::init_with_level(log::Level::Info).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    mount_to_body(|| view! { <App/> });
}

// TODO - Rearrange files

const NOTES: [(&str, &str); 30] = [
    ("A0", "https://tonejs.github.io/audio/salamander/A0.mp3"),
    ("C1", "https://tonejs.github.io/audio/salamander/C1.mp3"),
    ("D#1", "https://tonejs.github.io/audio/salamander/Ds1.mp3"),
    ("F#1", "https://tonejs.github.io/audio/salamander/Fs1.mp3"),
    ("A1", "https://tonejs.github.io/audio/salamander/A1.mp3"),
    ("C2", "https://tonejs.github.io/audio/salamander/C2.mp3"),
    ("D#2", "https://tonejs.github.io/audio/salamander/Ds2.mp3"),
    ("F#2", "https://tonejs.github.io/audio/salamander/Fs2.mp3"),
    ("A2", "https://tonejs.github.io/audio/salamander/A2.mp3"),
    ("C3", "https://tonejs.github.io/audio/salamander/C3.mp3"),
    ("D#3", "https://tonejs.github.io/audio/salamander/Ds3.mp3"),
    ("F#3", "https://tonejs.github.io/audio/salamander/Fs3.mp3"),
    ("A3", "https://tonejs.github.io/audio/salamander/A3.mp3"),
    ("C4", "https://tonejs.github.io/audio/salamander/C4.mp3"),
    ("D#4", "https://tonejs.github.io/audio/salamander/Ds4.mp3"),
    ("F#4", "https://tonejs.github.io/audio/salamander/Fs4.mp3"),
    ("A4", "https://tonejs.github.io/audio/salamander/A4.mp3"),
    ("C5", "https://tonejs.github.io/audio/salamander/C5.mp3"),
    ("D#5", "https://tonejs.github.io/audio/salamander/Ds5.mp3"),
    ("F#5", "https://tonejs.github.io/audio/salamander/Fs5.mp3"),
    ("A5", "https://tonejs.github.io/audio/salamander/A5.mp3"),
    ("C6", "https://tonejs.github.io/audio/salamander/C6.mp3"),
    ("D#6", "https://tonejs.github.io/audio/salamander/Ds6.mp3"),
    ("F#6", "https://tonejs.github.io/audio/salamander/Fs6.mp3"),
    ("A6", "https://tonejs.github.io/audio/salamander/A6.mp3"),
    ("C7", "https://tonejs.github.io/audio/salamander/C7.mp3"),
    ("D#7", "https://tonejs.github.io/audio/salamander/Ds7.mp3"),
    ("F#7", "https://tonejs.github.io/audio/salamander/Fs7.mp3"),
    ("A7", "https://tonejs.github.io/audio/salamander/A7.mp3"),
    ("C8", "https://tonejs.github.io/audio/salamander/C8.mp3"),
];

fn event_to_song_index(event: KeyboardEvent) -> Option<usize> {
    let has_modifier = event.meta_key() || event.ctrl_key() || event.shift_key() || event.alt_key();
    if has_modifier {
        return None;
    }

    // let song_index = "qwerasdfzxcvuiopjkl;m,./".find(event.key().as_str())?;
    let song_index = "qwertyuiopasdfghjkl;zxcvbnm,./".find(event.key().as_str())?;

    event.prevent_default();

    if event.repeat() {
        // Only send events on the initial keypress.
        // This needs to happen after we've prevented default.
        return None;
    }
    Some(song_index)
}

fn start_song_index(
    sampler: &mut Sampler,
    song: &Song,
    voices: &HashSet<usize>,
    song_index: usize,
) -> Vec<SamplerPlaybackGuard> {
    let mut sampler_playback_guards = Vec::new();
    let Some(slice) = &song.slices.get(song_index) else {
        return sampler_playback_guards;
    };
    for (voice, notes) in slice.notes_by_voice.iter().enumerate() {
        if !voices.contains(&voice) {
            continue;
        }
        for (&key, _) in notes.iter() {
            sampler_playback_guards.push(sampler.start_note(key.as_int() as i32).unwrap());
        }
    }

    sampler_playback_guards
}
