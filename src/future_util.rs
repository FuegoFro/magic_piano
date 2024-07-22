use js_sys::Promise;
use wasm_bindgen_futures::JsFuture;

pub trait PromiseAsFuture {
    fn into_future(self) -> JsFuture;
}

impl PromiseAsFuture for Promise {
    fn into_future(self) -> JsFuture {
        JsFuture::from(self)
    }
}
