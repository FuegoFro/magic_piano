use std::panic;

use leptos::prelude::*;
use leptos::mount::mount_to_body;

use crate::components::app::App;

mod components;
mod future_util;
mod html_util;
mod opensheetmusicdisplay_bindings;
mod playback_manager;
mod sampler;
mod song_data;

fn main() {
    console_log::init_with_level(log::Level::Info).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    mount_to_body(|| view! { <App /> });
}
