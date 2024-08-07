use std::collections::HashMap;

use itertools::Itertools;
use js_sys::JsString;
use leptos::{
    component, create_effect, create_node_ref, create_signal, document, view, window, CollectView,
    IntoView, Signal, SignalGet, SignalSet, SignalWith, WriteSignal,
};

use crate::components::keyboard_listener::LETTERS;
use crate::opensheetmusicdisplay_bindings::{CursorOptions, OpenSheetMusicDisplay};
use crate::song_data::SongData;

const KEY_HINT_CONTAINER_ID: &str = "magicPianoKeyHintContainer";

#[component]
pub fn SheetMusic(
    #[prop(into)] song_data: Signal<Option<SongData>>,
    #[prop(into)] start_cursor_index: Signal<usize>,
    #[prop(into)] current_cursor_index: Signal<usize>,
    #[prop(into)] osmd: Signal<Option<OpenSheetMusicDisplay>>,
    #[prop(into)] set_osmd: WriteSignal<Option<OpenSheetMusicDisplay>>,
) -> impl IntoView {
    let container_ref = create_node_ref();

    // Sync the indices to show to the cursor
    create_sync_cursor_effect(osmd, start_cursor_index, 1);
    create_sync_cursor_effect(osmd, current_cursor_index, 0);
    // When the start cursor is moved, update the key hints
    create_effect(move |previous_start_cursor_index| {
        let start_cursor_index = start_cursor_index.get();
        // Okay, this is definitely a bit cursed, but the outer Option<...> indicates whether or
        // not this effect has been run from Leptos' POV, and the inner Option<...> indicates
        // whether or not we were actually able to do the thing.
        if Some(Some(start_cursor_index)) == previous_start_cursor_index {
            return Some(start_cursor_index);
        }

        let letter_cursor_index_pairs = song_data.with(|song_data| {
            let song_data = song_data.as_ref()?;
            // First find the starting song index. This is kinda silly to have to look up, maybe
            // we should plumb it in directly.
            let start_song_index = song_data
                .slices
                .iter()
                .position(|s| s.cursor_index == start_cursor_index)?;
            Some(
                LETTERS
                    .chars()
                    .enumerate()
                    .flat_map(|(idx, c)| {
                        song_data
                            .slices
                            .get(start_song_index + idx)
                            .map(|s| (c, s.cursor_index))
                    })
                    .collect_vec(),
            )
        })?;

        let graphical_music_sheet = osmd.get()?.graphic()?;

        let letter_coords_id_pairs = letter_cursor_index_pairs
            .into_iter()
            .map(|(l, idx)| {
                let staff_entry = graphical_music_sheet.vertical_graphical_staff_entry_containers()
                    [idx]
                    .staff_entries()
                    .into_iter()
                    .next()
                    .unwrap();
                let x = staff_entry.position_and_shape().absolute_position().x();
                let y = staff_entry.get_highest_y_at_entry();
                let system_id = staff_entry.parent_measure().parent_music_system().id();
                (l, x, y, system_id)
            })
            .collect_vec();
        let y_by_system_id =
            letter_coords_id_pairs
                .iter()
                .fold(HashMap::new(), |mut map, (_, _, y, system_id)| {
                    let existing = map.entry(system_id).or_insert(*y);
                    // More-negative values are higher up, we want the highest up position.
                    *existing = existing.min(*y);
                    map
                });

        let svg = document().get_element_by_id("osmdSvgPage1")?;

        if let Some(existing_key_hint_container) =
            document().get_element_by_id(KEY_HINT_CONTAINER_ID)
        {
            existing_key_hint_container.remove();
        }

        let view = view! {
            <g id=KEY_HINT_CONTAINER_ID>

                {letter_coords_id_pairs.iter()
                    .map(|(l, x, _, system_id)| {
                        let x = x * 10. - 10.;
                        let y = y_by_system_id.get(system_id).unwrap() * 10. - 10.;
                        view! {
                            <circle cx={x + 5.} cy={y - 5.} r="15" fill="#99ef97"/>

                            <text
                                stroke-width="0.3"
                                fill="#000000"
                                stroke="none"
                                stroke-dasharray="none"
                                font-family="Times New Roman"
                                font-size="20px"
                                font-weight="normal"
                                font-style="normal"
                                x={x}
                                y={y}
                            >
                                {*l}
                            </text>
                        }
                    })
                    .collect_view()}

            </g>
        };
        svg.append_child(&view).unwrap();

        Some(start_cursor_index)
    });

    container_ref.on_load(move |container| {
        // Need this to traverse through multiple derefs correctly.
        let container: &web_sys::HtmlDivElement = &container;
        // let options = IOSMDOptions {
        //     page_format: None,
        //     drawing_parameters: None,
        // };
        // let options = serde_wasm_bindgen::to_value(&options).unwrap();
        // let osmd = OpenSheetMusicDisplay::new_with_options(container, options);
        let osmd = OpenSheetMusicDisplay::new(container);
        osmd.set_cursors_options(vec![
            CursorOptions::from_color("#33e02f".into())
                .to_js_value()
                .unwrap(),
            CursorOptions::from_color("#03f0fc".into())
                .to_js_value()
                .unwrap(),
        ]);
        js_sys::Reflect::set(&window(), &JsString::from("osmd"), &osmd).unwrap();
        set_osmd.set(Some(osmd));
    });

    view! { <div class="w-full h-full img-height-revert-layer" _ref=container_ref></div> }
}

fn create_sync_cursor_effect(
    osmd: Signal<Option<OpenSheetMusicDisplay>>,
    desired_index: Signal<usize>,
    nth_cursor: usize,
) {
    let (index_last_shown, set_index_last_shown) = create_signal(0);
    create_effect(move |_| {
        osmd.with(|osmd| {
            let to_show = desired_index.get() as i32;
            let diff = to_show - index_last_shown.get();
            let Some(cursor) = osmd
                .as_ref()
                .and_then(|osmd| osmd.cursors().into_iter().nth(nth_cursor))
            else {
                return;
            };
            if diff > 0 {
                for _ in 0..diff {
                    cursor.next();
                }
            } else {
                for _ in 0..(-diff) {
                    cursor.previous();
                }
            }
            set_index_last_shown.set(to_show);
        });
    });
}
