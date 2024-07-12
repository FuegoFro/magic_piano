use gloo::events::{EventListener, EventListenerOptions};
use gloo::utils::document;
use itertools::Itertools;
use log::{error, info};
use midly::num::{u4, u7};
use midly::{MidiMessage, Smf, TrackEventKind};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::panic;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use watchreload::start_reload_listener;
use web_sys::{AudioContext, Event, KeyboardEvent};

use crate::sampler::Sampler;

mod sampler;

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

    let ctx = AudioContext::new().unwrap();
    let sampler = Rc::new(RefCell::new(Sampler::new(ctx, &NOTES).await));
    let sampler2 = sampler.clone();

    let data = include_bytes!("../examples/A Million Stars.mid");
    let midi_file = Smf::parse(data).unwrap_or_else(|e| panic!("Unable to parse midi file: {e}"));
    let song = Song::from_smf(&midi_file);

    EventListener::new_with_options(
        &document(),
        "keydown",
        EventListenerOptions::enable_prevent_default(),
        move |event| {
            if let Some(midi_note) = event_to_midi_note(event) {
                if midi_note == 0 {
                    spawn_local(play_song(sampler.clone(), song.clone()));
                } else {
                    sampler.borrow_mut().start_note(midi_note).unwrap();
                }
            }
        },
    )
    .forget();

    EventListener::new_with_options(
        &document(),
        "keyup",
        EventListenerOptions::enable_prevent_default(),
        move |event| {
            if let Some(midi_note) = event_to_midi_note(event) {
                sampler2.borrow_mut().stop_note(midi_note).unwrap();
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

fn event_to_midi_note(event: &Event) -> Option<i32> {
    let event = event.dyn_ref::<KeyboardEvent>().unwrap();

    // info!("{} | {}", event.key(), event.repeat());
    let has_modifier = event.meta_key() || event.ctrl_key() || event.shift_key() || event.alt_key();
    if has_modifier {
        return None;
    }

    let midi_note = match event.key().as_ref() {
        "q" => 60,
        "2" => 61,
        "w" => 62,
        "3" => 63,
        "e" => 64,
        "r" => 65,
        "5" => 66,
        "t" => 67,
        "6" => 68,
        "y" => 69,
        "7" => 70,
        "u" => 71,
        "i" => 72,
        "b" => 0,
        _ => return None,
    };

    event.prevent_default();

    if event.repeat() {
        // Only send events on the initial keypress.
        // This needs to happen after we've prevented default.
        return None;
    }
    Some(midi_note)
}

async fn play_song(sampler: Rc<RefCell<Sampler>>, song: Song) {
    for slice in song.slices.iter() {
        for (_voice, notes) in slice.notes_by_voice.iter().enumerate() {
            for (&key, _) in notes.iter() {
                sampler
                    .borrow_mut()
                    .start_note(key.as_int() as i32)
                    .unwrap();
                // let live_event = LiveEvent::Midi {
                //     channel: u4::new(voice as u8),
                //     message: MidiMessage::NoteOn { key, vel },
                // };
                // buf.clear();
                // live_event.write_std(&mut buf).unwrap();
                // conn_out.send(&buf).unwrap();
            }
        }

        gloo::timers::future::TimeoutFuture::new(1000).await;
        // thread::sleep(Duration::from_secs(1));

        for (_voice, notes) in slice.notes_by_voice.iter().enumerate() {
            for (&key, _) in notes.iter() {
                sampler.borrow_mut().stop_note(key.as_int() as i32).unwrap();

                // let live_event = LiveEvent::Midi {
                //     channel: u4::new(voice as u8),
                //     message: MidiMessage::NoteOff {
                //         key,
                //         vel: u7::new(0),
                //     },
                // };
                // buf.clear();
                // live_event.write_std(&mut buf).unwrap();
                // conn_out.send(&buf).unwrap();
            }
        }

        gloo::timers::future::TimeoutFuture::new(1000).await;
        // thread::sleep(Duration::from_secs(1));
    }
}

#[derive(Clone)]
struct Song {
    voices: usize,
    slices: Vec<TimeSlice>,
}

impl Song {
    fn from_smf(smf: &Smf) -> Self {
        let mut voice_keys = HashSet::new();

        // Put all messages into a single vec sorted by position
        // Keep track of everything turned on so far. Maybe just clone the notes vec when time advances and apply changes to the latest vec.
        struct VoiceAndMessage {
            /// Track and channel
            voice_key: (usize, u4),
            message: MidiMessage,
        }

        // Make sure the song starts at 0
        let mut messages_by_position = HashMap::from([(0, Vec::new())]);

        for (track_idx, track) in smf.tracks.iter().enumerate() {
            let mut position = 0;
            for event in track {
                position += event.delta.as_int();
                let TrackEventKind::Midi { channel, message } = event.kind else {
                    continue;
                };
                let voice_key = (track_idx, channel);
                voice_keys.insert(voice_key);

                messages_by_position
                    .entry(position)
                    .or_insert_with(Vec::new)
                    .push(VoiceAndMessage { voice_key, message })
            }
        }

        let voice_by_key = voice_keys
            .into_iter()
            .sorted()
            .enumerate()
            .map(|(idx, key)| (key, idx))
            .collect::<HashMap<_, _>>();

        let mut current_slice = TimeSlice::empty(voice_by_key.len());
        let slices = messages_by_position
            .keys()
            .sorted()
            .map(|position| {
                for message in messages_by_position[position].iter() {
                    let notes_current_voice = current_slice
                        .notes_by_voice
                        .get_mut(voice_by_key[&message.voice_key])
                        .unwrap();
                    match message.message {
                        MidiMessage::NoteOn { key, vel } => {
                            if vel > 0 {
                                notes_current_voice.insert(key, vel);
                            } else {
                                notes_current_voice.remove(&key);
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            // TODO - use vel?
                            notes_current_voice.remove(&key);
                        }
                        _ => {}
                    }
                }
                current_slice.clone()
            })
            .collect();

        Self {
            voices: voice_by_key.len(),
            slices,
        }
    }
}

#[derive(Clone)]
struct TimeSlice {
    notes_by_voice: Vec<HashMap<u7, u7>>,
}

impl TimeSlice {
    fn empty(num_voices: usize) -> Self {
        Self {
            notes_by_voice: vec![HashMap::default(); num_voices],
        }
    }
}
