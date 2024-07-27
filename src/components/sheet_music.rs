use js_sys::JsString;
use leptos::{
    component, create_effect, create_node_ref, create_signal, view, window, IntoView, ReadSignal,
    Signal, SignalGet, SignalSet, SignalWith, WriteSignal,
};

use crate::opensheetmusicdisplay_bindings::OpenSheetMusicDisplay;

#[component]
pub fn SheetMusic(
    #[prop(into)] index_to_show: Signal<usize>,
    #[prop(into)] osmd: ReadSignal<Option<OpenSheetMusicDisplay>>,
    #[prop(into)] set_osmd: WriteSignal<Option<OpenSheetMusicDisplay>>,
) -> impl IntoView {
    let container_ref = create_node_ref();
    let (index_last_shown, set_index_last_shown) = create_signal(0);

    // Sync the index to show to the cursor
    create_effect(move |_| {
        osmd.with(|osmd| {
            let to_show = index_to_show.get() as i32;
            let diff = to_show - index_last_shown.get();
            let Some(cursor) = osmd.as_ref().and_then(|osmd| osmd.cursor()) else {
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
        js_sys::Reflect::set(&window(), &JsString::from("osmd"), &osmd).unwrap();
        set_osmd.set(Some(osmd));
    });

    view! { <div class="w-full h-full img-height-revert-layer" _ref=container_ref></div> }
}
