use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use bit_set::BitSet;
use fraction::Fraction;
use itertools::Itertools;

use crate::opensheetmusicdisplay_bindings::{OpenSheetMusicDisplay, Tie};

#[derive(Clone)]
pub struct SongData {
    // TODO - Use this when constructing voice states
    #[allow(dead_code)]
    pub voices: usize,
    pub slices: Vec<TimeSlice>,
}

impl Debug for SongData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SongData")
            .field("..", &"..".to_string())
            .finish()
    }
}

#[derive(Hash, Eq, PartialEq)]
struct TieKey {
    start_time: Fraction,
    voice: u32,
    pitch: u32,
    duration: Fraction,
}

impl TieKey {
    fn from_tie(tie: &Tie) -> Self {
        let measure_position = tie
            .start_note()
            .source_measure()
            .absolute_timestamp()
            .to_rust_fraction()
            .unwrap();
        let within_measure_timestamp = tie
            .start_note()
            .voice_entry()
            .timestamp()
            .to_rust_fraction()
            .unwrap();
        Self {
            start_time: measure_position + within_measure_timestamp,
            voice: tie.start_note().voice_entry().parent_voice().voice_id(),
            pitch: tie.pitch().half_tone(),
            duration: tie.duration().to_rust_fraction().unwrap(),
        }
    }
}

impl SongData {
    pub fn from_osmd(osmd: &OpenSheetMusicDisplay) -> Self {
        // Build the (staff_id, voice_id) pairs and sort them. Use position as final voice index.
        let voice_keys = osmd
            .sheet()
            .staves()
            .into_iter()
            .flat_map(|staff| {
                let id = staff.id_in_music_sheet();
                staff.voices().into_iter().map(move |v| (id, v.voice_id()))
            })
            .unique()
            .sorted()
            .collect_vec();
        let cursor = osmd.cursor().unwrap();
        cursor.reset();

        // Iterate through to build the slices.
        // Store (voice, active pitch, end timestamp)
        // Ignore subsequent items in ties (use start time/voice/pitch/duration as key?)
        //  where time is voice entry timestamp + measure absolute timestamp?
        let mut active_notes_by_voice: Vec<Vec<(u32, Fraction)>> =
            (0..voice_keys.len()).map(|_| Vec::new()).collect_vec();
        let mut seen_ties = HashSet::new();
        let mut slices = Vec::new();
        let mut cursor_index = 0;
        while !cursor.iterator().end_reached() {
            let current_timestamp = cursor
                .iterator()
                .current_timestamp()
                .to_rust_fraction()
                .unwrap();
            // First expire any old notes
            active_notes_by_voice = active_notes_by_voice
                .into_iter()
                .map(|voice_notes| {
                    voice_notes
                        .into_iter()
                        .filter(|(_, end_timestamp)| end_timestamp > &current_timestamp)
                        .collect_vec()
                })
                .collect_vec();

            // Then add any new notes
            let mut added_new_note = false;
            let Some(current_voice_entries) = cursor.iterator().current_voice_entries() else {
                continue;
            };
            for voice_entry in current_voice_entries {
                let voice_key = (
                    voice_entry
                        .parent_source_staff_entry()
                        .parent_staff()
                        .id_in_music_sheet(),
                    voice_entry.parent_voice().voice_id(),
                );
                let voice = voice_keys
                    .iter()
                    .position(|k| *k == voice_key)
                    .unwrap_or_else(|| panic!("Unable to find voice index for key {voice_key:?}"));
                for note in voice_entry.notes() {
                    let pitch = if let Some(pitch) = note.pitch() {
                        pitch.half_tone() + 12
                    } else {
                        // A rest, ignore
                        continue;
                    };
                    let duration = if let Some(tie) = note.tie() {
                        let tie_key = TieKey::from_tie(&tie);
                        if !seen_ties.insert(tie_key) {
                            continue;
                        }
                        tie.duration().to_rust_fraction().unwrap()
                    } else {
                        note.length().to_rust_fraction().unwrap()
                    };
                    active_notes_by_voice[voice].push((pitch, current_timestamp + duration));
                    added_new_note = true;
                }
            }

            // If anything was added, copy all the notes over to the slice. This avoids treating
            // note-stops as a new slice, including skipping rests.
            if added_new_note {
                slices.push(TimeSlice::new(
                    active_notes_by_voice
                        .iter()
                        .map(|voice| voice.iter().map(|(pitch, _)| *pitch as usize).collect())
                        .collect_vec(),
                    cursor_index,
                ));
            }

            cursor.next();
            cursor_index += 1;
        }

        Self {
            voices: voice_keys.len(),
            slices,
        }
    }
}

#[derive(Clone)]
pub struct TimeSlice {
    pub notes_by_voice: Vec<BitSet>,
    pub cursor_index: usize,
}

impl TimeSlice {
    fn new(notes_by_voice: Vec<BitSet>, cursor_index: usize) -> Self {
        Self {
            notes_by_voice,
            cursor_index,
        }
    }
}
