use std::fmt::Debug;

use bit_set::BitSet;
use wasm_bindgen::JsValue;
use web_sys::{AudioContext, GainNode};

use crate::sampler::{Sampler, SamplerPlaybackGuard};
use crate::song_data::SongData;

/// This is the bridge between the UI and the playback layer. While it still uses some UI concepts,
/// eg leptos' `Signal`s and `Resource`s, the APIs it exposes are all in the playback parlance (eg
/// gain is set as a float multiplier, rather than a 0-100 volume).
///
/// Will panic if any JS operation fails for some reason.
#[derive(Debug)]
pub struct PlaybackManager {
    sampler: Sampler,
    overall_gain: GainNode,
    voice_gains: Vec<GainNode>,
    song_data: Option<SongData>,
}

impl PlaybackManager {
    pub async fn initialize() -> Self {
        let ctx = AudioContext::new().unwrap();
        let sampler = Sampler::initialize(ctx.clone(), &NOTES).await;

        let (overall_gain, voice_gains) =
            Self::create_gains(&ctx).expect("Unable to create playback gain nodes");

        Self {
            sampler,
            overall_gain,
            voice_gains,
            song_data: None,
        }
    }

    fn create_gains(ctx: &AudioContext) -> Result<(GainNode, Vec<GainNode>), JsValue> {
        let overall_gain = ctx.create_gain()?;
        overall_gain.connect_with_audio_node(&ctx.destination())?;

        let voice_gains = (0..4)
            .map(|_| -> Result<GainNode, JsValue> {
                let voice_gain = ctx.create_gain()?;
                voice_gain.connect_with_audio_node(&overall_gain)?;
                Ok(voice_gain)
            })
            .collect::<Result<Vec<GainNode>, JsValue>>()?;

        Ok((overall_gain, voice_gains))
    }

    pub fn set_song_data(&mut self, song_data: SongData) {
        self.song_data = Some(song_data);
    }

    pub fn set_voice_gain(&self, voice: usize, gain: f32) {
        if let Some(voice_gain) = self.voice_gains.get(voice) {
            voice_gain.gain().set_value(gain);
        }
    }

    pub fn set_overall_gain(&self, gain: f32) {
        self.overall_gain.gain().set_value(gain);
    }

    pub fn start_notes_at_relative_index(
        &self,
        song_index: usize,
        active_voices: &BitSet,
    ) -> Option<(usize, Vec<SamplerPlaybackGuard>)> {
        let slice = self.song_data.as_ref()?.slices.get(song_index)?;
        let mut sampler_playback_guards = Vec::new();

        for (voice, notes) in slice.notes_by_voice.iter().enumerate() {
            if !active_voices.contains(voice) {
                continue;
            }
            let Some(voice_gain) = self.voice_gains.get(voice) else {
                continue;
            };

            for key in notes.iter() {
                sampler_playback_guards
                    .push(self.sampler.start_note(key as i32, voice_gain).unwrap());
            }
        }

        Some((slice.cursor_index, sampler_playback_guards))
    }

    pub fn max_song_index(&self) -> Option<usize> {
        Some(self.song_data.as_ref()?.slices.len() - 1)
    }

    pub fn cursor_index_for_song_index(&self, song_index: usize) -> Option<usize> {
        Some(
            self.song_data
                .as_ref()?
                .slices
                .get(song_index)?
                .cursor_index,
        )
    }
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
