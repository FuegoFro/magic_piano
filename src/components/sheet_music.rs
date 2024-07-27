use js_sys::JsString;
use leptos::{
    component, create_effect, create_node_ref, create_signal, view, window, IntoView, Signal,
    SignalGet, SignalSet, SignalWith, WriteSignal,
};

use crate::opensheetmusicdisplay_bindings::{CursorOptions, OpenSheetMusicDisplay};

#[component]
pub fn SheetMusic(
    #[prop(into)] start_cursor_index: Signal<usize>,
    #[prop(into)] current_cursor_index: Signal<usize>,
    #[prop(into)] osmd: Signal<Option<OpenSheetMusicDisplay>>,
    #[prop(into)] set_osmd: WriteSignal<Option<OpenSheetMusicDisplay>>,
) -> impl IntoView {
    let container_ref = create_node_ref();

    // Sync the indices to show to the cursor
    create_sync_cursor_effect(osmd, start_cursor_index, 1);
    create_sync_cursor_effect(osmd, current_cursor_index, 0);

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
