use crate::sampler::Sampler;
use gloo::events::{EventListener, EventListenerOptions};
use gloo::net::http::Request;
use gloo::utils::document;
use log::{error, info};
use midly::Smf;
use song::Song;
use std::cell::{RefCell, RefMut};
use std::collections::HashSet;
use std::panic;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use watchreload::start_reload_listener;
use web_sys::{AudioContext, Event, KeyboardEvent};

mod sampler;
mod song;

#[wasm_bindgen]
pub async fn start() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    match start_reload_listener() {
        Ok(true) => {
            info!("Reload listener started.");
        }
        Ok(false) => (),
        Err(e) => {
            error!("Could not start reload listener: {:?}", e);
        }
    }

    struct State {
        sampler: RefCell<Sampler>,
        song: Song,
        voices: HashSet<usize>,
    }
    let state = {
        let sampler = Sampler::new(AudioContext::new().unwrap(), &NOTES).await;

        let data = Request::get("examples/Mam'selle.mid")
            .send()
            .await
            .unwrap()
            .binary()
            .await
            .unwrap();
        let midi_file =
            Smf::parse(&data).unwrap_or_else(|e| panic!("Unable to parse midi file: {e}"));
        let song = Song::from_smf(&midi_file);
        let voices = get_voices(&song);

        Rc::new(State {
            sampler: RefCell::new(sampler),
            song,
            voices,
        })
    };

    EventListener::new_with_options(
        &document(),
        "keydown",
        EventListenerOptions::enable_prevent_default(),
        {
            // Binding for the closure
            let state = state.clone();
            move |event| {
                if let Some(song_index) = event_to_song_index(event) {
                    start_song_index(
                        state.sampler.borrow_mut(),
                        &state.song,
                        &state.voices,
                        song_index,
                    );
                }
            }
        },
    )
    .forget();

    EventListener::new_with_options(
        &document(),
        "keyup",
        EventListenerOptions::enable_prevent_default(),
        {
            // Binding for the closure, no need to clone
            let state = state;
            move |event| {
                if let Some(song_index) = event_to_song_index(event) {
                    stop_song_index(
                        state.sampler.borrow_mut(),
                        &state.song,
                        &state.voices,
                        song_index,
                    );
                }
            }
        },
    )
    .forget();

    info!("Hello world 2");
}

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

fn get_voices(song: &Song) -> HashSet<usize> {
    (0..song.voices).collect()
}

fn event_to_song_index(event: &Event) -> Option<usize> {
    let event = event.dyn_ref::<KeyboardEvent>().unwrap();

    let has_modifier = event.meta_key() || event.ctrl_key() || event.shift_key() || event.alt_key();
    if has_modifier {
        return None;
    }

    let song_index = "qwerasdfzxcvuiopjkl;m,./".find(event.key().as_str())?;

    event.prevent_default();

    if event.repeat() {
        // Only send events on the initial keypress.
        // This needs to happen after we've prevented default.
        return None;
    }
    Some(song_index)
}

fn start_song_index(
    mut sampler: RefMut<Sampler>,
    song: &Song,
    voices: &HashSet<usize>,
    song_index: usize,
) {
    let Some(slice) = &song.slices.get(song_index) else {
        return;
    };
    for (voice, notes) in slice.notes_by_voice.iter().enumerate() {
        if !voices.contains(&voice) {
            continue;
        }
        for (&key, _) in notes.iter() {
            sampler.start_note(key.as_int() as i32).unwrap();
        }
    }
}

fn stop_song_index(
    mut sampler: RefMut<Sampler>,
    song: &Song,
    voices: &HashSet<usize>,
    song_index: usize,
) {
    let Some(slice) = &song.slices.get(song_index) else {
        return;
    };
    for (voice, notes) in slice.notes_by_voice.iter().enumerate() {
        if !voices.contains(&voice) {
            continue;
        }
        for (&key, _) in notes.iter() {
            sampler.stop_note(key.as_int() as i32).unwrap();
        }
    }
}
