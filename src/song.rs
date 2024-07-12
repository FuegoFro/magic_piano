use itertools::Itertools;
use midly::num::{u4, u7};
use midly::{MidiMessage, Smf, TrackEventKind};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct Song {
    pub voices: usize,
    pub slices: Vec<TimeSlice>,
}

impl Song {
    pub fn from_smf(smf: &Smf) -> Self {
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
pub struct TimeSlice {
    pub notes_by_voice: Vec<HashMap<u7, u7>>,
}

impl TimeSlice {
    fn empty(num_voices: usize) -> Self {
        Self {
            notes_by_voice: vec![HashMap::default(); num_voices],
        }
    }
}
