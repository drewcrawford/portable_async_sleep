// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
WebAssembly implementation using the host's `setTimeout` API.

Timeout callbacks deallocate themselves after invocation. Long durations are
split both because JavaScript timers use a signed 32-bit millisecond delay and
so callbacks from cancelled futures are promptly eligible to run and clean up.
*/

use std::time::Duration;
use wasm_bindgen::prelude::*;

const MAX_TIMEOUT_MILLIS: u128 = 60_000;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(callback: &js_sys::Function, millis: i32);
}

fn duration_millis_ceil(duration: Duration) -> u128 {
    let millis = duration.as_millis();
    if duration.subsec_nanos() % 1_000_000 == 0 {
        millis
    } else {
        millis + 1
    }
}

async fn sleep_millis(millis: i32) {
    let (sender, receiver) = r#continue::continuation();
    let callback = Closure::once_into_js(move || sender.send(()));
    set_timeout(callback.unchecked_ref(), millis);

    // setTimeout now owns the JavaScript reference. Dropping our handle before
    // awaiting keeps the returned future Send; the one-shot callback releases
    // its Rust allocation when the host invokes it.
    drop(callback);

    receiver.await;
}

pub async fn async_sleep(duration: Duration) {
    let mut remaining_millis = duration_millis_ceil(duration);

    while remaining_millis > 0 {
        let millis = remaining_millis.min(MAX_TIMEOUT_MILLIS);
        sleep_millis(millis as i32).await;
        remaining_millis -= millis;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn sub_millisecond_duration_rounds_up() {
        assert_eq!(duration_millis_ceil(Duration::from_nanos(1)), 1);
        assert_eq!(duration_millis_ceil(Duration::from_micros(999)), 1);
        assert_eq!(duration_millis_ceil(Duration::from_micros(1_001)), 2);
    }

    #[wasm_bindgen_test]
    fn long_duration_does_not_wrap_the_timeout() {
        let millis = duration_millis_ceil(Duration::MAX);
        let first_timeout = millis.min(MAX_TIMEOUT_MILLIS) as i32;

        assert_eq!(first_timeout, MAX_TIMEOUT_MILLIS as i32);
        assert!(millis > MAX_TIMEOUT_MILLIS);
    }

    #[wasm_bindgen_test]
    async fn cancelled_timeout_callback_is_safe() {
        let mut sleep = Box::pin(async_sleep(Duration::from_millis(10)));
        assert!(futures::poll!(sleep.as_mut()).is_pending());
        drop(sleep);

        // Allow the abandoned callback to run and release its Rust allocation.
        async_sleep(Duration::from_millis(20)).await;
    }
}
