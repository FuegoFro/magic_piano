#![allow(dead_code, unused_imports, unused_variables)]

mod sampler;

use std::collections::{HashMap, HashSet};
use std::time::Duration;
use std::{env, fs, panic, thread};

use crate::sampler::Sampler;
use gloo::events::{EventListener, EventListenerOptions};
use gloo::net::http::Request;
use gloo::utils::document;
use itertools::Itertools;
use js_sys::{ArrayBuffer, Uint8Array};
use log::{error, info};
use midly::live::LiveEvent;
use midly::num::{u4, u7};
use midly::{MidiMessage, Smf, TrackEventKind};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use watchreload::start_reload_listener;
use web_sys::{AudioBuffer, AudioContext, Event, HtmlButtonElement, KeyboardEvent, OscillatorType};

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
    let sampler = Sampler::new(&ctx, &NOTES).await;

    let button = document().get_element_by_id("test").unwrap();
    EventListener::new(&button, "click", |_| spawn_local(do_sound_wrapper())).forget();

    EventListener::new_with_options(
        &document(),
        "keydown",
        EventListenerOptions::enable_prevent_default(),
        move |event| {
            if let Some(midi_note) = event_to_midi_note(event) {
                sampler.start_note(&ctx, midi_note).unwrap();
            }
        },
    )
    .forget();

    // EventListener::new_with_options(
    //     &document(),
    //     "keyup",
    //     EventListenerOptions::enable_prevent_default(),
    //     move |event| {
    //         if let Some(midi_note) = event_to_midi_note(event) {
    //             stop_note(midi_note);
    //         }
    //     },
    // )
    // .forget();

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
    let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap();

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

fn start_note(midi_note: u32) {}

/*
urls: {
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
},
release: 1,
baseUrl: "https://tonejs.github.io/audio/salamander/",

*/

async fn do_sound_wrapper() {
    do_sound().await.unwrap()
}

async fn do_sound() -> Result<(), JsValue> {
    let ctx = AudioContext::new()?;
    let frequency = 493.883;
    let start_time = ctx.current_time();
    let duration = 0.116;

    // for i in 1..=5 {
    //     let i = i as f32;
    //     play_note(&ctx, frequency * i, 0.1 / (11.0 - i), start_time, 2.0)?;
    //     // play_note(&ctx, frequency * 5.0, 0.01, start_time, 2.0)?;
    //     info!("Done setting up {i}");
    // }

    play_sample(
        &ctx,
        "https://tonejs.github.io/audio/salamander/C4.mp3",
        2.0,
    )
    .await?;

    // play_note(&ctx, 493.883, 0.1, start_time, duration)?;
    // play_note(&ctx, 659.255, 0.1, start_time + duration, duration)?;

    Ok(())
}

fn play_note(
    ctx: &AudioContext,
    frequency: f32,
    volume: f32,
    start_time: f64,
    duration: f64,
) -> Result<(), JsValue> {
    let osc1 = ctx.create_oscillator()?;
    let osc2 = ctx.create_oscillator()?;

    // osc1.frequency().set_value(frequency + 1.0);
    // osc2.frequency().set_value(frequency - 2.0);
    osc1.frequency().set_value(frequency);
    osc2.frequency().set_value(frequency);

    osc1.set_type(OscillatorType::Triangle);
    osc2.set_type(OscillatorType::Triangle);
    // osc1.set_type(OscillatorType::Sine);
    // osc2.set_type(OscillatorType::Sine);

    let gain = ctx.create_gain()?;
    gain.gain().set_value(volume);

    osc1.connect_with_audio_node(&gain)?;
    osc2.connect_with_audio_node(&gain)?;

    gain.connect_with_audio_node(&ctx.destination())?;

    let stop_time = start_time + duration;

    osc1.start_with_when(start_time)?;
    osc2.start_with_when(start_time)?;

    osc1.stop_with_when(stop_time)?;
    osc2.stop_with_when(stop_time)?;

    let ramp_duration = 0.05;
    gain.gain().set_value_at_time(0.0, start_time)?;
    gain.gain()
        .linear_ramp_to_value_at_time(volume, start_time + ramp_duration)?;
    gain.gain()
        .set_value_at_time(volume, stop_time - ramp_duration)?;
    gain.gain().linear_ramp_to_value_at_time(0.0, stop_time)?;

    Ok(())
}

async fn play_sample(ctx: &AudioContext, url: &str, playback_rate: f32) -> Result<(), JsValue> {
    let audio_data = Request::get(url)
        .send()
        .await
        .unwrap()
        .binary()
        .await
        .unwrap();
    let audio_data = Uint8Array::from(audio_data.as_slice()).buffer();

    let buffer = JsFuture::from(ctx.decode_audio_data(&audio_data)?)
        .await?
        .dyn_into::<AudioBuffer>()?;

    let buffer_source = ctx.create_buffer_source()?;
    buffer_source.set_buffer(Some(&buffer));

    buffer_source.connect_with_audio_node(&ctx.destination())?;
    buffer_source.playback_rate().set_value(playback_rate);
    buffer_source.start()?;

    Ok(())
}

fn main() {
    let Some(path) = env::args().nth(1) else {
        panic!("Expected to be passed file location")
    };

    let data = fs::read(&path).unwrap_or_else(|e| panic!("Unable to read data from {path}: {e}"));
    let midi_file = Smf::parse(&data).unwrap_or_else(|e| panic!("Unable to parse midi file: {e}"));

    // let midi_out = MidiOutput::new("My Test Output").unwrap();
    //
    // // Get an output port (read from console if multiple are available)
    // let out_ports = midi_out.ports();
    // let out_port: &MidiOutputPort = match out_ports.len() {
    //     0 => panic!("no output midi port found"),
    //     1 => {
    //         println!(
    //             "Choosing the only available output port: {}",
    //             midi_out.port_name(&out_ports[0]).unwrap()
    //         );
    //         &out_ports[0]
    //     }
    //     _ => {
    //         println!("\nAvailable output ports:");
    //         for (i, p) in out_ports.iter().enumerate() {
    //             println!("{}: {}", i, midi_out.port_name(p).unwrap());
    //         }
    //         panic!();
    //         // print!("Please select output port: ");
    //         // stdout().flush()?;
    //         // let mut input = String::new();
    //         // stdin().read_line(&mut input)?;
    //         // out_ports
    //         //     .get(input.trim().parse::<usize>()?)
    //         //     .ok_or("invalid output port selected")?
    //     }
    // };
    //
    // println!("\nOpening connection");
    // let mut conn_out = midi_out.connect(out_port, "midir-test").unwrap();

    // println!("{:?}", &midi_file.header);
    // println!("Has {} tracks", midi_file.tracks.len());
    // for track in midi_file.tracks.iter() {
    //     println!("---");
    //     println!("---");
    //     for event in track {
    //         // println!("{}: {}", event.delta, kind_str(&event.kind));
    //         // println!("{}: {:?}", event.delta, &event.kind);
    //         println!("{event:?}");
    //     }
    // }

    let song = Song::from_smf(&midi_file);
    for slice in song.slices.iter() {
        println!("{:?}", slice.notes_by_voice);
    }

    // let mut buf = Vec::new();
    for slice in song.slices.iter() {
        for (voice, notes) in slice.notes_by_voice.iter().enumerate() {
            for (&key, &vel) in notes.iter() {
                // let live_event = LiveEvent::Midi {
                //     channel: u4::new(voice as u8),
                //     message: MidiMessage::NoteOn { key, vel },
                // };
                // buf.clear();
                // live_event.write_std(&mut buf).unwrap();
                // conn_out.send(&buf).unwrap();
            }
        }
        thread::sleep(Duration::from_secs(1));

        for (voice, notes) in slice.notes_by_voice.iter().enumerate() {
            for (&key, _) in notes.iter() {
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
        thread::sleep(Duration::from_secs(1));
    }
}

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
