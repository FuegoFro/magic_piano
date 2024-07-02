use std::collections::{HashMap, HashSet};
use std::time::Duration;
use std::{env, fs, thread};

use itertools::Itertools;
use midir::{MidiOutput, MidiOutputPort};
use midly::live::LiveEvent;
use midly::num::{u4, u7};
use midly::{MidiMessage, Smf, TrackEventKind};

fn main() {
    let Some(path) = env::args().nth(1) else {
        panic!("Expected to be passed file location")
    };

    let data = fs::read(&path).unwrap_or_else(|e| panic!("Unable to read data from {path}: {e}"));
    let midi_file = Smf::parse(&data).unwrap_or_else(|e| panic!("Unable to parse midi file: {e}"));

    let midi_out = MidiOutput::new("My Test Output").unwrap();

    // Get an output port (read from console if multiple are available)
    let out_ports = midi_out.ports();
    let out_port: &MidiOutputPort = match out_ports.len() {
        0 => panic!("no output midi port found"),
        1 => {
            println!(
                "Choosing the only available output port: {}",
                midi_out.port_name(&out_ports[0]).unwrap()
            );
            &out_ports[0]
        }
        _ => {
            println!("\nAvailable output ports:");
            for (i, p) in out_ports.iter().enumerate() {
                println!("{}: {}", i, midi_out.port_name(p).unwrap());
            }
            panic!();
            // print!("Please select output port: ");
            // stdout().flush()?;
            // let mut input = String::new();
            // stdin().read_line(&mut input)?;
            // out_ports
            //     .get(input.trim().parse::<usize>()?)
            //     .ok_or("invalid output port selected")?
        }
    };

    println!("\nOpening connection");
    let mut conn_out = midi_out.connect(out_port, "midir-test").unwrap();

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

    let mut buf = Vec::new();
    for slice in song.slices.iter() {
        for (voice, notes) in slice.notes_by_voice.iter().enumerate() {
            for (&key, &vel) in notes.iter() {
                let live_event = LiveEvent::Midi {
                    channel: u4::new(voice as u8),
                    message: MidiMessage::NoteOn { key, vel },
                };
                buf.clear();
                live_event.write_std(&mut buf).unwrap();
                conn_out.send(&buf).unwrap();
            }
        }
        thread::sleep(Duration::from_secs(1));

        for (voice, notes) in slice.notes_by_voice.iter().enumerate() {
            for (&key, _) in notes.iter() {
                let live_event = LiveEvent::Midi {
                    channel: u4::new(voice as u8),
                    message: MidiMessage::NoteOff {
                        key,
                        vel: u7::new(0),
                    },
                };
                buf.clear();
                live_event.write_std(&mut buf).unwrap();
                conn_out.send(&buf).unwrap();
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
