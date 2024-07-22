use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};

use futures::future::join_all;
use gloo::net::http::Request;
use itertools::Itertools;
use js_sys::Uint8Array;
use log::error;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, AudioNode, GainNode};

use crate::future_util::PromiseAsFuture;

/// Holds onto the playback nodes that were started by the `Sampler` allowing you to stop them
/// before they reach the end of the sample.
///
/// Dropping the object will stop the playback.
// TODO - If we need to get around this behavior we can add a `.forget()`.
pub struct SamplerPlaybackGuard {
    ctx: AudioContext,
    buffer_source: AudioBufferSourceNode,
    gain: GainNode,
}

impl SamplerPlaybackGuard {
    fn stop_playback(&self) -> Result<(), JsValue> {
        let current_time = self.ctx.current_time();
        let end_time = current_time + 1.0;
        self.gain.gain().set_value_at_time(1.0, current_time)?;
        self.gain
            .gain()
            .linear_ramp_to_value_at_time(0.0, end_time)?;
        self.buffer_source.stop_with_when(end_time)?;

        Ok(())
    }
}

impl Drop for SamplerPlaybackGuard {
    fn drop(&mut self) {
        if let Err(e) = self.stop_playback() {
            error!("Failed to stop playback: {e:?}");
        }
    }
}

/// Heavily inspired by https://tonejs.github.io/docs/latest/classes/Sampler
pub struct Sampler {
    ctx: AudioContext,
    buffers: BTreeMap<i32, AudioBuffer>,
}

impl Debug for Sampler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sampler")
            .field("..", &"..".to_string())
            .finish()
    }
}

impl Sampler {
    /// * `urls`: A series of (note_name, url) pairs
    // fn new(urls: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>) -> Self {
    pub async fn initialize(ctx: AudioContext, urls: &[(&str, &str)]) -> Self {
        let ctx_ref = &ctx;

        let buffer_futures = urls.iter().map(|(note_name, url)| async move {
            (
                note_name_to_midi_note(note_name)
                    .unwrap_or_else(|| panic!("Malformed note name {note_name}")),
                url_to_audio_buffer(ctx_ref, url)
                    .await
                    .unwrap_or_else(|e| panic!("Unable to fetch sample at {url}: {e:?}")),
            )
        });
        let buffers = join_all(buffer_futures)
            .await
            .into_iter()
            .collect::<BTreeMap<_, _>>();

        Self { ctx, buffers }
    }

    pub fn start_note(
        &self,
        midi_note: i32,
        output_node: &AudioNode,
    ) -> Result<SamplerPlaybackGuard, JsValue> {
        // Find closest note
        let above = self.buffers.range(midi_note..).next();
        let below = self.buffers.range(..=midi_note).last();
        let (diff, buffer) = [above, below]
            .into_iter()
            .flatten()
            .map(|(key, buffer)| (midi_note - key, buffer))
            .min_by_key(|(diff, _)| diff.abs())
            .unwrap_or_else(|| {
                panic!("Unable to find a corresponding buffer for midi note {midi_note}")
            });

        // Pitch shift accordingly
        let buffer_source = self.ctx.create_buffer_source()?;
        buffer_source.set_buffer(Some(buffer));

        let frequency_ratio = 2f32.powf(diff as f32 / 12.0);
        buffer_source.playback_rate().set_value(frequency_ratio);

        // Set up our gain (for fadeout at the end) and play
        let gain = self.ctx.create_gain()?;
        gain.gain().set_value(1.0);

        buffer_source.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(output_node)?;

        buffer_source.start()?;

        Ok(SamplerPlaybackGuard {
            ctx: self.ctx.clone(),
            buffer_source,
            gain,
        })
    }
}

fn note_name_to_midi_note(note_name: &str) -> Option<i32> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        RegexBuilder::new(r"^([a-g](?:b|#|##|x|bb|###|#x|x#|bbb)?)(-?[0-9]+)")
            .case_insensitive(true)
            .build()
            .unwrap()
    });

    let captures = RE.captures(note_name)?;
    let (pitch_name, octave_str) = captures
        .iter()
        .skip(1)
        .map(|m| m.map(|m| m.as_str()))
        .collect_tuple()?;
    let (pitch_name, octave_str) = (pitch_name?, octave_str?);

    let scale_index = pitch_name_to_scale_index(pitch_name)?;
    let octave = octave_str.parse::<i32>().ok()?;

    Some(((octave + 1) * 12) + scale_index)
}

fn pitch_name_to_scale_index(pitch_name: &str) -> Option<i32> {
    Some(match pitch_name.to_lowercase().as_str() {
        "cbbb" => -3,
        "cbb" => -2,
        "cb" => -1,
        "c" => 0,
        "c#" => 1,
        "cx" => 2,
        "c##" => 2,
        "c###" => 3,
        "cx#" => 3,
        "c#x" => 3,
        "dbbb" => -1,
        "dbb" => 0,
        "db" => 1,
        "d" => 2,
        "d#" => 3,
        "dx" => 4,
        "d##" => 4,
        "d###" => 5,
        "dx#" => 5,
        "d#x" => 5,
        "ebbb" => 1,
        "ebb" => 2,
        "eb" => 3,
        "e" => 4,
        "e#" => 5,
        "ex" => 6,
        "e##" => 6,
        "e###" => 7,
        "ex#" => 7,
        "e#x" => 7,
        "fbbb" => 2,
        "fbb" => 3,
        "fb" => 4,
        "f" => 5,
        "f#" => 6,
        "fx" => 7,
        "f##" => 7,
        "f###" => 8,
        "fx#" => 8,
        "f#x" => 8,
        "gbbb" => 4,
        "gbb" => 5,
        "gb" => 6,
        "g" => 7,
        "g#" => 8,
        "gx" => 9,
        "g##" => 9,
        "g###" => 10,
        "gx#" => 10,
        "g#x" => 10,
        "abbb" => 6,
        "abb" => 7,
        "ab" => 8,
        "a" => 9,
        "a#" => 10,
        "ax" => 11,
        "a##" => 11,
        "a###" => 12,
        "ax#" => 12,
        "a#x" => 12,
        "bbbb" => 8,
        "bbb" => 9,
        "bb" => 10,
        "b" => 11,
        "b#" => 12,
        "bx" => 13,
        "b##" => 13,
        "b###" => 14,
        "bx#" => 14,
        "b#x" => 14,
        _ => return None,
    })
}

async fn url_to_audio_buffer(ctx: &AudioContext, url: &str) -> Result<AudioBuffer, JsValue> {
    let audio_data = Request::get(url)
        .send()
        .await
        .unwrap()
        .binary()
        .await
        .unwrap();
    let audio_data = Uint8Array::from(audio_data.as_slice()).buffer();

    ctx.decode_audio_data(&audio_data)?
        .into_future()
        .await?
        .dyn_into::<AudioBuffer>()
}
