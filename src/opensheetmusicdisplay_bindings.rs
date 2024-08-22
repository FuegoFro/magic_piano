use std::fmt::Debug;

use js_sys::{JsString, Promise};
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
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
    pub fn cursors(this: &OpenSheetMusicDisplay) -> Vec<Cursor>;

    #[wasm_bindgen(method, setter, js_name = "cursorsOptions")]
    pub fn set_cursors_options(this: &OpenSheetMusicDisplay, cursor_options: Vec<JsValue>);

    #[wasm_bindgen(method, js_name = "enableOrDisableCursors")]
    pub fn enable_or_disable_cursors(this: &OpenSheetMusicDisplay, enabled: bool);

    #[wasm_bindgen(method, getter)]
    pub fn sheet(this: &OpenSheetMusicDisplay) -> MusicSheet;

    #[wasm_bindgen(method, getter)]
    pub fn graphic(this: &OpenSheetMusicDisplay) -> Option<GraphicalMusicSheet>;

    #[wasm_bindgen(method, js_name = "handleResize")]
    pub fn handle_resize(this: &OpenSheetMusicDisplay, start: &JsValue, end: &JsValue);

    #[wasm_bindgen(method, getter)]
    pub fn rules(this: &OpenSheetMusicDisplay) -> EngravingRules;

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

    pub type GraphicalMusicSheet;

    #[wasm_bindgen(method, getter, js_name = "verticalGraphicalStaffEntryContainers")]
    pub fn vertical_graphical_staff_entry_containers(
        this: &GraphicalMusicSheet,
    ) -> Vec<VerticalGraphicalStaffEntryContainer>;

    #[wasm_bindgen(method, getter, js_name = "musicPages")]
    pub fn music_pages(this: &GraphicalMusicSheet) -> Vec<GraphicalMusicPage>;

    pub type VerticalGraphicalStaffEntryContainer;

    // NOTE! This actually returns a Vec<Option<GraphicalStaffEntry>> but we can't nest the
    // generics like that.
    #[wasm_bindgen(method, getter, js_name = "staffEntries")]
    pub fn staff_entries(this: &VerticalGraphicalStaffEntryContainer) -> Vec<GraphicalStaffEntry>;

    pub type GraphicalStaffEntry;

    #[wasm_bindgen(method, getter, js_name = "PositionAndShape")]
    pub fn position_and_shape(this: &GraphicalStaffEntry) -> BoundingBox;

    #[wasm_bindgen(method, js_name = "getHighestYAtEntry")]
    pub fn get_highest_y_at_entry(this: &GraphicalStaffEntry) -> f32;

    #[wasm_bindgen(method, getter, js_name = "parentMeasure")]
    pub fn parent_measure(this: &GraphicalStaffEntry) -> GraphicalMeasure;

    #[wasm_bindgen(method, getter, js_name = "graphicalVoiceEntries")]
    pub fn graphical_voice_entries(this: &GraphicalStaffEntry) -> Vec<GraphicalVoiceEntry>;

    #[wasm_bindgen(method, getter, js_name = "graphicalTies")]
    pub fn graphical_ties(this: &GraphicalStaffEntry) -> Vec<GraphicalTie>;

    pub type GraphicalVoiceEntry;

    #[wasm_bindgen(method, getter, js_name = "parentVoiceEntry")]
    pub fn parent_voice_entry(this: &GraphicalVoiceEntry) -> VoiceEntry;

    #[wasm_bindgen(method, getter)]
    pub fn notes(this: &GraphicalVoiceEntry) -> Vec<GraphicalNote>;

    pub type GraphicalNote;

    #[wasm_bindgen(method, getter, js_name = "sourceNote")]
    pub fn source_note(this: &GraphicalNote) -> Note;

    #[wasm_bindgen(method, js_name = "getSVGGElement")]
    pub fn get_svg_g_element(this: &GraphicalNote) -> HtmlElement;

    #[wasm_bindgen(method, js_name = "getStemSVG")]
    pub fn get_stem_svg(this: &GraphicalNote) -> HtmlElement;

    #[wasm_bindgen(method, js_name = "getBeamSVGs")]
    pub fn get_beam_svgs(this: &GraphicalNote) -> Vec<HtmlElement>;

    pub type GraphicalTie;

    #[wasm_bindgen(method, getter, js_name = "startNote")]
    pub fn start_note(this: &GraphicalTie) -> GraphicalNote;

    #[wasm_bindgen(method, getter, js_name = "SVGElement")]
    pub fn svg_element(this: &GraphicalTie) -> HtmlElement;

    pub type GraphicalMeasure;

    #[wasm_bindgen(method, getter, js_name = "parentMusicSystem")]
    pub fn parent_music_system(this: &GraphicalMeasure) -> MusicSystem;

    pub type GraphicalMusicPage;

    #[wasm_bindgen(method, getter, js_name = "musicSystems")]
    pub fn music_systems(this: &GraphicalMusicPage) -> Vec<MusicSystem>;

    pub type GraphicalSlur;

    #[wasm_bindgen(method, getter, js_name = "SVGElement")]
    pub fn svg_element(this: &GraphicalSlur) -> HtmlElement;

    #[wasm_bindgen(method, getter)]
    pub fn slur(this: &GraphicalSlur) -> Slur;

    pub type MusicSystem;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &MusicSystem) -> u32;

    #[wasm_bindgen(method, getter, js_name = "staffLines")]
    pub fn staff_lines(this: &MusicSystem) -> Vec<StaffLine>;

    pub type StaffLine;

    #[wasm_bindgen(method, getter, js_name = "graphicalSlurs")]
    pub fn graphical_slurs(this: &StaffLine) -> Vec<GraphicalSlur>;

    pub type BoundingBox;

    #[wasm_bindgen(method, getter, js_name = "absolutePosition")]
    pub fn absolute_position(this: &BoundingBox) -> PointF2D;

    pub type PointF2D;

    #[wasm_bindgen(method, getter)]
    pub fn x(this: &PointF2D) -> f32;

    #[wasm_bindgen(method, getter)]
    pub fn y(this: &PointF2D) -> f32;

    pub type EngravingRules;

    #[wasm_bindgen(method, setter, js_name = "NewSystemAtXMLNewPageAttribute")]
    pub fn set_new_system_at_xml_new_page_attribute(this: &EngravingRules, enabled: bool);

    #[wasm_bindgen(method, setter, js_name = "NewSystemAtXMLNewSystemAttribute")]
    pub fn set_new_system_at_xml_new_system_attribute(this: &EngravingRules, enabled: bool);

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

    #[wasm_bindgen(method, getter, js_name = "cursorElement")]
    pub fn cursor_element(this: &Cursor) -> HtmlElement;

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

    #[wasm_bindgen(method, js_name = "isRest")]
    pub fn is_rest(this: &Note) -> bool;

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

    pub type Slur;

    #[wasm_bindgen(method, getter, js_name = "StartNote")]
    pub fn start_note(this: &Slur) -> Note;

    pub type Fraction;

    #[wasm_bindgen(method, getter)]
    pub fn numerator(this: &Fraction) -> u32;

    #[wasm_bindgen(method, getter)]
    pub fn denominator(this: &Fraction) -> u32;

    #[wasm_bindgen(method, getter, js_name = "wholeValue")]
    pub fn whole_value(this: &Fraction) -> u32;
}

impl Clone for OpenSheetMusicDisplay {
    fn clone(&self) -> Self {
        JsValue::clone(self).dyn_into::<Self>().unwrap()
    }
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

#[derive(Serialize)]
pub struct CursorOptions {
    /// alpha value to be used with color (0.0 transparent, 0.5 medium, 1.0 opaque).
    pub alpha: f32,
    /// Color to draw the cursor (eg hex, including `#`)
    pub color: String,
    /// If true, this cursor will be followed.
    pub follow: bool,
    #[serde(rename = "type")]
    pub cursor_type: CursorType,
}

impl CursorOptions {
    pub fn from_color(color: String) -> Self {
        Self {
            alpha: 0.5,
            color,
            follow: true,
            cursor_type: CursorType::CurrentNotes,
        }
    }

    pub fn to_js_value(&self) -> Result<JsValue, serde_wasm_bindgen::Error> {
        serde_wasm_bindgen::to_value(self)
    }
}

#[allow(dead_code)]
#[derive(Serialize)]
pub enum CursorType {
    /// Standard highlighting current notes
    CurrentNotes = 0,
    /// Thin line left to the current notes
    LeftOfNotes = 1,
    /// Short thin line on top of stave and left to the current notes
    LeftAndTopOfNotes = 2,
    /// Current measure
    CurrentMeasure = 3,
    /// Current measure to left of current notes
    LeftMeasure = 4,
}
