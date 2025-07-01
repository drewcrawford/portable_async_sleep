//SPDX-License-Identifier: MIT OR Apache-2.0
/*!
WASM implementation using browser's setTimeout API.

This implementation leverages JavaScript's event loop and Promise API
to provide async sleep functionality in WebAssembly environments.
*/

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

// Use JavaScript's global setTimeout function which works in both browser and Node.js
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(closure: &js_sys::Function, millis: i32);
}

pub async fn async_sleep(duration: std::time::Duration) {
    let millis = duration.as_millis() as i32;
    
    // Create a JavaScript Promise that resolves after the timeout
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        // Use the global setTimeout which works in both browser and Node.js
        set_timeout(&resolve, millis);
    });
    
    // Convert the JS Promise to a Rust Future and await it
    JsFuture::from(promise).await.expect("timer promise failed");
}