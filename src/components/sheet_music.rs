use std::collections::HashMap;

use crate::components::keyboard_listener::LETTERS;
use crate::opensheetmusicdisplay_bindings::{CursorOptions, OpenSheetMusicDisplay};
use crate::song_data::SongData;
use itertools::Itertools;
use js_sys::JsString;
use leptos::{
    component, create_effect, create_node_ref, create_signal, create_trigger, document, view,
    window, CollectView, IntoView, Signal, SignalGet, SignalSet, SignalWith, WriteSignal,
};
use wasm_bindgen::closure::Closure;
use web_sys::{ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition};

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
    let on_render = create_trigger();

    // Sync the indices to show to the cursor
    create_sync_cursor_effect(osmd, start_cursor_index, 1);
    create_sync_cursor_effect(osmd, current_cursor_index, 0);
    // When the start cursor is moved, update the key hints
    create_effect(move |_| {
        let start_cursor_index = start_cursor_index.get();
        // Rerun this if we re-render
        on_render.track();

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
                let (x, y, system_id) = graphical_music_sheet
                    .vertical_graphical_staff_entry_containers()[idx]
                    .staff_entries()
                    .into_iter()
                    .filter(|se| !se.is_undefined())
                    .map(|se| {
                        (
                            se.position_and_shape().absolute_position().x(),
                            se.get_highest_y_at_entry(),
                            se.parent_measure().parent_music_system().id(),
                        )
                    })
                    // We want the upper-left-most x/y coords
                    .reduce(|(x_a, y_a, id_a), (x_b, y_b, id_b)| {
                        assert_eq!(
                            id_a, id_b,
                            "Vertical entry had staves with different system ids!"
                        );
                        (x_a.min(x_b), y_a.min(y_b), id_a)
                    })
                    .unwrap();
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

                {letter_coords_id_pairs
                    .iter()
                    .map(|(l, x, _, system_id)| {
                        let x = x * 10. - 10.;
                        let y = y_by_system_id.get(system_id).unwrap() * 10. - 10.;
                        view! {
                            <circle cx=x + 5. cy=y - 5. r="15" fill="#99ef97"></circle>

                            <text
                                stroke-width="0.3"
                                fill="#000000"
                                stroke="none"
                                stroke-dasharray="none"
                                font-family="Times New Roman"
                                font-size="20px"
                                font-weight="normal"
                                font-style="normal"
                                x=x
                                y=y
                            >
                                {*l}
                            </text>
                        }
                    })
                    .collect_view()}

            </g>
        };
        svg.append_child(&view).unwrap();

        Some(())
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

        let start: Box<dyn Fn()> = Box::new(|| {});
        let end: Box<dyn Fn()> = Box::new(move || on_render.notify());

        osmd.handle_resize(
            &Closure::wrap(start).into_js_value(),
            &Closure::wrap(end).into_js_value(),
        );
        js_sys::Reflect::set(&window(), &JsString::from("osmd"), &osmd).unwrap();
        set_osmd.set(Some(osmd));
    });

    let zoom_input_node_ref = create_node_ref();
    let (zoom_value, set_zoom_value) = create_signal(50f64);
    let original_linebreaks_node_ref = create_node_ref();

    view! {
        <div class="flex flex-row space-x-1">
            <p>"Zoom: "</p>
            <input
                _ref=zoom_input_node_ref
                type="range"
                min="25"
                max="200"
                step="25"
                list="zoom_values"
                prop:value=zoom_value
                on:input=move |_| {
                    if let Some(zoom_input) = zoom_input_node_ref.get() {
                        set_zoom_value.set(zoom_input.value_as_number());
                    }
                }

                on:change=move |_| {
                    osmd.with(move |osmd| {
                        let Some(osmd) = osmd.as_ref() else { return };
                        osmd.set_zoom((zoom_value.get() / 100f64) as f32);
                        osmd.render();
                        on_render.notify();
                    });
                }
            />

            <datalist id="zoom_values">
                {(25..=200).step_by(25).map(|v| view! { <option value=v></option> }).collect_view()}
            </datalist>
            <p>{zoom_value} "%"</p>
        </div>
        <div class="flex flex-row items-baseline space-x-1">
            <p>"Use original line breaks:"</p>
            <input
                _ref=original_linebreaks_node_ref
                type="checkbox"
                on:change=move |_| {
                    if let Some(original_linebreaks_input) = original_linebreaks_node_ref.get() {
                        let use_original_linebreaks = original_linebreaks_input.checked();
                        osmd.with(move |osmd| {
                            let Some(osmd) = osmd.as_ref() else { return };
                            osmd.rules().set_new_system_at_xml_new_page_attribute(use_original_linebreaks);
                            osmd.rules().set_new_system_at_xml_new_system_attribute(use_original_linebreaks);
                            osmd.render();
                            on_render.notify();
                        });
                    }
                }
            />
        </div>
        <div
            class="w-full h-full img-height-revert-layer img-scroll-margin-block-5em"
            _ref=container_ref
        ></div>
    }
}

fn create_sync_cursor_effect(
    osmd: Signal<Option<OpenSheetMusicDisplay>>,
    desired_index: Signal<usize>,
    nth_cursor: usize,
) {
    let (index_last_shown, set_index_last_shown) = create_signal(0);
    create_effect(move |_| {
        // Important that these happen before we might early-exit
        let to_show = desired_index.get() as i32;
        let index_last_shown = index_last_shown.get();

        if to_show == index_last_shown {
            return;
        }

        osmd.with(|osmd| {
            let Some(cursor) = osmd
                .as_ref()
                .and_then(|osmd| osmd.cursors().into_iter().nth(nth_cursor))
            else {
                return;
            };

            let diff = if to_show < 20 && index_last_shown > 50 {
                // Special case when resetting near the beginning of a long song
                cursor.reset();
                to_show
            } else {
                to_show - index_last_shown
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

            cursor
                .cursor_element()
                .scroll_into_view_with_scroll_into_view_options(
                    ScrollIntoViewOptions::new()
                        .behavior(ScrollBehavior::Instant)
                        .block(ScrollLogicalPosition::Nearest),
                );
            set_index_last_shown.set(to_show);
        });
    });
}
