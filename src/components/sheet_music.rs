use std::collections::HashMap;

use bit_set::BitSet;
use itertools::Itertools;
use js_sys::JsString;
use leptos::{
    component, create_effect, create_node_ref, create_signal, create_trigger, document,
    spawn_local, view, window, with, CollectView, IntoView, ReadSignal, Resource, Signal,
    SignalGet, SignalGetUntracked, SignalSet, SignalWith, SignalWithUntracked, WriteSignal,
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition};

use crate::components::app::SongChoice;
use crate::components::keyboard_listener::LETTERS;
use crate::future_util::PromiseAsFuture;
use crate::html_util::HtmlCollectionIntoIterator;
use crate::opensheetmusicdisplay_bindings::{CursorOptions, OpenSheetMusicDisplay};
use crate::song_data::SongData;

const KEY_HINT_CONTAINER_ID: &str = "magicPianoKeyHintContainer";

#[component]
pub fn SheetMusic(
    #[prop(into)] active_voices: Signal<BitSet>,
    #[prop(into)] song_data: Signal<Option<SongData>>,
    #[prop(into)] start_cursor_index: Signal<usize>,
    #[prop(into)] current_cursor_index: Signal<usize>,
    #[prop(into)] song_raw_data: Resource<SongChoice, JsString>,
    #[prop(into)] set_song_data: WriteSignal<Option<SongData>>,
) -> impl IntoView {
    let (osmd, set_osmd) = create_signal::<Option<OpenSheetMusicDisplay>>(None);
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

    // Sync the note colors with the active voices
    create_effect(move |_| {
        on_render.track();
        with!(|osmd, song_data, active_voices| {
            let song_data = song_data.as_ref()?;
            // We pull elements to re-color from a bunch of different places (unfortunately,
            // OSMD doesn't make this easy for us). Each of the `*_elements` iterators below
            // ends up being an iterator of `(active, element): (bool, HtmlElement)`. The comments
            // just before each of them is how to access the voice entry used to compute `active`
            // and the `element` itself, respectively, from JS.

            // osmd.graphic.musicPages[x].musicSystems[x].staffLines[x].graphicalSlurs[x].slur.startNote.voiceEntry
            // osmd.graphic.musicPages[x].musicSystems[x].staffLines[x].graphicalSlurs[x].SVGElement
            let slur_elements = osmd
                .as_ref()?
                .graphic()?
                .music_pages()
                .into_iter()
                .flat_map(|pages| pages.music_systems().into_iter())
                .flat_map(|systems| systems.staff_lines().into_iter())
                .flat_map(|lines| lines.graphical_slurs().into_iter())
                .map(|graphical_slur| {
                    let active = active_voices.contains(
                        song_data.voice_index_mapping.index_for_voice_entry(
                            &graphical_slur.slur().start_note().voice_entry(),
                        ),
                    );
                    (active, graphical_slur.svg_element())
                });

            // osmd.graphic.verticalGraphicalStaffEntryContainers[x].staffEntries[x].graphicalTies[x].startNote.sourceNote.voiceEntry
            // osmd.graphic.verticalGraphicalStaffEntryContainers[x].staffEntries[x].graphicalTies[x].SVGElement
            let tie_elements = osmd
                .as_ref()?
                .graphic()?
                .vertical_graphical_staff_entry_containers()
                .into_iter()
                .flat_map(|vgsec| vgsec.staff_entries().into_iter())
                .filter(|se| !se.is_undefined())
                .flat_map(|graphical_staff_entry| {
                    graphical_staff_entry.graphical_ties().into_iter()
                })
                .map(|graphical_tie| {
                    let active = active_voices.contains(
                        song_data.voice_index_mapping.index_for_voice_entry(
                            &graphical_tie.start_note().source_note().voice_entry(),
                        ),
                    );
                    (active, graphical_tie.svg_element())
                });

            // osmd.graphic.verticalGraphicalStaffEntryContainers[x].staffEntries[x].graphicalVoiceEntries[x].parentVoiceEntry
            // osmd.graphic.verticalGraphicalStaffEntryContainers[x].staffEntries[x].graphicalVoiceEntries[x].notes[x].getSVGGElement().getElementsByTagName("path")
            let note_beam_stem_elements = osmd
                .as_ref()?
                .graphic()?
                .vertical_graphical_staff_entry_containers()
                .into_iter()
                .flat_map(|vgsec| vgsec.staff_entries().into_iter())
                .filter(|se| !se.is_undefined())
                .flat_map(|graphical_staff_entry| {
                    graphical_staff_entry.graphical_voice_entries().into_iter()
                })
                .flat_map(|graphical_voice_entry| {
                    let active = active_voices.contains(
                        song_data
                            .voice_index_mapping
                            .index_for_voice_entry(&graphical_voice_entry.parent_voice_entry()),
                    );
                    graphical_voice_entry
                        .notes()
                        .into_iter()
                        // Get all the elements that might be or have paths we want to color
                        .flat_map(|graphical_note| {
                            let g_element = graphical_note.get_svg_g_element();
                            let stem_element = graphical_note.get_stem_svg();
                            let beam_elements = graphical_note.get_beam_svgs();

                            beam_elements.into_iter().chain([g_element, stem_element])
                        })
                        .map(move |element| (active, element))
                });

            // Now that we have our `(active, element)` pairs, get the `<path>` tags from the
            // elements and color them accordingly.
            slur_elements
                .chain(tie_elements)
                .chain(note_beam_stem_elements)
                .filter(|(_, element)| !element.is_undefined() && !element.is_null())
                // Extract the path elements/children
                .flat_map(|(active, element)| {
                    if &element.tag_name() == "path" {
                        vec![(active, element.dyn_into::<web_sys::Element>().unwrap())]
                    } else {
                        element
                            .get_elements_by_tag_name("path")
                            .into_iter()
                            .map(|e| (active, e))
                            .collect_vec()
                    }
                })
                // Finally, color it!
                .for_each(|(active, path_element)| {
                    let color = if active { "#000000" } else { "#aaaaaa" };
                    let stroke = path_element.get_attribute("stroke");
                    // If we are Some and not "none"
                    if stroke.map(|stroke| stroke != "none").unwrap_or(false) {
                        path_element.set_attribute("stroke", color).unwrap()
                    }
                    let fill = path_element.get_attribute("fill");
                    if fill.map(|fill| fill != "none").unwrap_or(false) {
                        path_element.set_attribute("fill", color).unwrap()
                    }
                });

            Some(()) // (Function returns `Option` so we can conveniently use `?`)
        });
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
    let (zoom_value, set_zoom_value) = create_signal(75f64);
    let original_linebreaks_node_ref = create_node_ref();

    // Load the song into osmd and extract the SongData.
    // Not using create_local_resource because we depend on osmd which doesn't impl Eq, which is
    // needed for Resource inputs.
    create_effect(move |_| {
        // Important that we track the usage of both of these before we enter `spawn_local`
        let load_promise = with!(|osmd, song_raw_data| {
            let Some(osmd) = osmd else { return None };
            let Some(song_raw_data) = song_raw_data else {
                return None;
            };
            Some(osmd.load(song_raw_data))
        });
        let Some(load_promise) = load_promise else {
            return;
        };
        spawn_local(async move {
            load_promise.into_future().await.unwrap();
            // We tracked this usage above
            osmd.with_untracked(|osmd| {
                // We shouldn't be able to get here without this set.
                let osmd = osmd.as_ref().unwrap();
                osmd.set_zoom((zoom_value.get_untracked() / 100f64) as f32);
                osmd.render();
                // Now load the song data
                let cursor = osmd.cursor().unwrap();
                cursor.reset();
                cursor.hide();
                let song_data = SongData::from_osmd(osmd);
                set_song_data.set(Some(song_data));
                // Reset and show the cursors
                for cursor in osmd.cursors() {
                    cursor.reset();
                    cursor.show();
                }

                // Do this once we've set the song data
                on_render.notify();
            });
        });
    });

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
                            osmd.rules()
                                .set_new_system_at_xml_new_page_attribute(use_original_linebreaks);
                            osmd.rules()
                                .set_new_system_at_xml_new_system_attribute(
                                    use_original_linebreaks,
                                );
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
    osmd: ReadSignal<Option<OpenSheetMusicDisplay>>,
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
