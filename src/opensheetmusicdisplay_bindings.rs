use std::fmt::Debug;

use js_sys::{JsString, Promise};
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

#[wasm_bindgen(js_namespace = opensheetmusicdisplay)]
extern "C" {
    pub type OpenSheetMusicDisplay;

    #[wasm_bindgen(constructor)]
    pub fn new(container: &HtmlElement) -> OpenSheetMusicDisplay;

    #[wasm_bindgen(constructor)]
    pub fn new_with_options(container: &HtmlElement, options: JsValue) -> OpenSheetMusicDisplay;

    #[wasm_bindgen(method, getter)]
    pub fn version(this: &OpenSheetMusicDisplay) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn zoom(this: &OpenSheetMusicDisplay) -> f32;

    #[wasm_bindgen(method, setter)]
    pub fn set_zoom(this: &OpenSheetMusicDisplay, zoom: f32);

    #[wasm_bindgen(method)]
    pub fn load(this: &OpenSheetMusicDisplay, content: &JsString) -> Promise;

    #[wasm_bindgen(method)]
    pub fn render(this: &OpenSheetMusicDisplay);

    #[wasm_bindgen(method, getter)]
    pub fn cursor(this: &OpenSheetMusicDisplay) -> Option<Cursor>;

    #[wasm_bindgen(method, getter)]
    pub fn sheet(this: &OpenSheetMusicDisplay) -> MusicSheet;

    pub type MusicSheet;

    #[wasm_bindgen(method, getter)]
    pub fn staves(this: &MusicSheet) -> Vec<Staff>;

    pub type SourceStaffEntry;

    #[wasm_bindgen(method, getter, js_name = "parentStaff")]
    pub fn parent_staff(this: &SourceStaffEntry) -> Staff;

    pub type Staff;

    #[wasm_bindgen(method, getter, js_name = "idInMusicSheet")]
    pub fn id_in_music_sheet(this: &Staff) -> u32;

    #[wasm_bindgen(method, getter)]
    pub fn voices(this: &Staff) -> Vec<Voice>;

    pub type SourceMeasure;

    #[wasm_bindgen(method, getter, js_name = "absoluteTimestamp")]
    pub fn absolute_timestamp(this: &SourceMeasure) -> Fraction;

    pub type Cursor;

    #[wasm_bindgen(method)]
    pub fn reset(this: &Cursor);

    #[wasm_bindgen(method)]
    pub fn show(this: &Cursor);

    #[wasm_bindgen(method)]
    pub fn hide(this: &Cursor);

    #[wasm_bindgen(method)]
    pub fn next(this: &Cursor);

    #[wasm_bindgen(method)]
    pub fn previous(this: &Cursor);

    #[wasm_bindgen(method)]
    pub fn update(this: &Cursor);

    #[wasm_bindgen(method, getter)]
    pub fn iterator(this: &Cursor) -> MusicPartManagerIterator;

    pub type MusicPartManagerIterator;

    #[wasm_bindgen(method, getter, js_name = "endReached")]
    pub fn end_reached(this: &MusicPartManagerIterator) -> bool;

    #[wasm_bindgen(method, getter, js_name = "currentVoiceEntries")]
    pub fn current_voice_entries(this: &MusicPartManagerIterator) -> Option<Vec<VoiceEntry>>;

    #[wasm_bindgen(method, getter, js_name = "currentTimeStamp")]
    pub fn current_timestamp(this: &MusicPartManagerIterator) -> Fraction;

    pub type VoiceEntry;

    #[wasm_bindgen(method, getter)]
    pub fn notes(this: &VoiceEntry) -> Vec<Note>;

    #[wasm_bindgen(method, getter, js_name = "stemDirection")]
    pub fn stem_direction(this: &VoiceEntry) -> StemDirectionType;

    #[wasm_bindgen(method, getter, js_name = "parentVoice")]
    pub fn parent_voice(this: &VoiceEntry) -> Voice;

    #[wasm_bindgen(method, getter, js_name = "parentSourceStaffEntry")]
    pub fn parent_source_staff_entry(this: &VoiceEntry) -> SourceStaffEntry;

    #[wasm_bindgen(method, getter)]
    pub fn timestamp(this: &VoiceEntry) -> Fraction;

    pub type Voice;

    #[wasm_bindgen(method, getter, js_name = "voiceId")]
    pub fn voice_id(this: &Voice) -> u32;

    pub type Note;

    #[wasm_bindgen(method, getter)]
    pub fn length(this: &Note) -> Fraction;

    #[wasm_bindgen(method, getter)]
    pub fn pitch(this: &Note) -> Option<Pitch>;

    #[wasm_bindgen(method, getter)]
    pub fn tie(this: &Note) -> Option<Tie>;

    #[wasm_bindgen(method, getter, js_name = "voiceEntry")]
    pub fn voice_entry(this: &Note) -> VoiceEntry;

    #[wasm_bindgen(method, getter, js_name = "sourceMeasure")]
    pub fn source_measure(this: &Note) -> SourceMeasure;

    pub type Pitch;

    #[wasm_bindgen(method, getter)]
    pub fn frequency(this: &Pitch) -> f32;

    #[wasm_bindgen(method, getter, js_name = "halfTone")]
    pub fn half_tone(this: &Pitch) -> u32;

    pub type Tie;

    #[wasm_bindgen(method, getter, js_name = "Duration")]
    pub fn duration(this: &Tie) -> Fraction;

    #[wasm_bindgen(method, getter, js_name = "Pitch")]
    pub fn pitch(this: &Tie) -> Pitch;

    #[wasm_bindgen(method, getter, js_name = "StartNote")]
    pub fn start_note(this: &Tie) -> Note;

    pub type Fraction;

    #[wasm_bindgen(method, getter)]
    pub fn numerator(this: &Fraction) -> u32;

    #[wasm_bindgen(method, getter)]
    pub fn denominator(this: &Fraction) -> u32;

    #[wasm_bindgen(method, getter, js_name = "wholeValue")]
    pub fn whole_value(this: &Fraction) -> u32;
}

#[wasm_bindgen]
#[derive(Debug)]
pub enum StemDirectionType {
    // wasm_bindgen won't let us put negative values here even though they're valid :/
    // Undefined = -1,
    Up = 0,
    Down = 1,
    None = 2,
    Double = 3,
}

impl Fraction {
    pub fn to_rust_fraction(&self) -> Option<fraction::Fraction> {
        let js_value: &JsValue = self;
        if js_value.is_undefined() {
            return None;
        }
        Some(
            fraction::Fraction::new(self.numerator(), self.denominator())
                + fraction::Fraction::from(self.whole_value()),
        )
    }
}

#[derive(Serialize, Default)]
pub struct IOSMDOptions {
    #[serde(rename = "pageFormat")]
    pub page_format: Option<PageFormat>,
    #[serde(rename = "drawingParameters")]
    pub drawing_parameters: Option<DrawingParameters>,
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Serialize)]
pub enum PageFormat {
    A3_L,
    A3_P,
    A4_L,
    A4_P,
    A5_L,
    A5_P,
    A6_L,
    A6_P,
    Endless,
    Letter_L,
    Letter_P,
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Serialize)]
pub enum DrawingParameters {
    allon,
    compact,
    compacttight,
    default,
    leadsheet,
    preview,
    thumbnail,
}
