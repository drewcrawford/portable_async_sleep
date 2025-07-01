//SPDX-License-Identifier: MIT OR Apache-2.0
/*!
WASM implementation using browser's setTimeout API.

This implementation leverages JavaScript's event loop and the continue crate
to provide async sleep functionality in WebAssembly environments with Send futures.
*/

use wasm_bindgen::prelude::*;

// Use JavaScript's global setTimeout function which works in both browser and Node.js
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(closure: &js_sys::Function, millis: i32);
}

pub async fn async_sleep(duration: std::time::Duration) {
    let millis = duration.as_millis() as i32;
    
    // Create a continuation pair for Send-safe async communication
    let (sender, receiver) = r#continue::continuation();
    
    // Create and immediately use the closure in a scope to ensure it's dropped
    {
        // Create a closure that will send the signal when the timeout fires
        let callback = Closure::once(move || {
            sender.send(());
        });
        
        // Schedule the timeout
        set_timeout(callback.as_ref().unchecked_ref(), millis);
        
        // Leak the closure to prevent it from being dropped too early
        // JavaScript will hold the reference until the timeout fires
        callback.forget();
    }
    
    // Await the receiver - this future is Send
    receiver.await;
}