//SPDX-License-Identifier: MIT OR Apache-2.0
/*!

![logo](../../../art/logo.png)

A portable async sleep function for Rust.

This crate provides a runtime-agnostic implementation of async sleep functionality.
Unlike runtime-specific sleep functions (e.g., `tokio::time::sleep` or `async_std::task::sleep`),
`portable_async_sleep` works with any async runtime or executor.

# Features

- **Runtime-agnostic**: Works with tokio, async-std, smol, or any other async runtime
- **Lightweight**: Uses standard library primitives with minimal overhead
- **Thread-safe**: Can be used from multiple async tasks simultaneously
- **Accurate timing**: Respects the requested sleep duration

# Implementation

The default implementation uses a dedicated background thread that manages sleep timers using
standard library channels and timeouts. This approach ensures maximum compatibility across
all async runtimes while maintaining good performance.

While the stdlib-based implementation is the most portable, the crate is designed to support
alternative backends in the future that may offer better performance or integration with
specific runtimes, while still maintaining the same portable API.

# Examples

Basic usage:

```
use portable_async_sleep::async_sleep;
use std::time::Duration;

# use test_executors::async_test;
# #[async_test]
# async fn test() {
async_sleep(Duration::from_millis(100)).await;
println!("Slept for 100ms!");
# }
```

Using with concurrent tasks:

```
use portable_async_sleep::async_sleep;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

# use test_executors::async_test;
# #[async_test]
# async fn test() {
let start = Instant::now();

// Start two sleep operations concurrently
let sleep1 = async_sleep(Duration::from_millis(100));
let sleep2 = async_sleep(Duration::from_millis(200));

// Wait for both to complete
futures::join!(sleep1, sleep2);

// Total time should be ~200ms, not 300ms
let elapsed = start.elapsed();
assert!(elapsed >= Duration::from_millis(200));
assert!(elapsed < Duration::from_millis(250));
# }
```
*/

#[cfg(not(target_arch = "wasm32"))]
mod stdlib;

#[cfg(target_arch = "wasm32")]
mod wasm;

/// Asynchronously sleeps for the specified duration.
///
/// This function suspends the current async task for at least the specified duration.
/// The actual sleep time may be slightly longer due to OS scheduling and timer precision,
/// but will never be shorter than the requested duration.
///
/// The current implementation uses standard library primitives for maximum portability,
/// but future versions may support alternative backends for improved efficiency while
/// maintaining the same API.
///
/// # Arguments
///
/// * `duration` - The minimum duration to sleep for
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use portable_async_sleep::async_sleep;
/// use std::time::Duration;
///
/// # use test_executors::async_test;
/// # #[async_test]
/// # async fn test() {
/// // Sleep for 500 milliseconds
/// async_sleep(Duration::from_millis(500)).await;
/// # }
/// ```
///
/// Measuring sleep accuracy:
///
/// ```
/// use portable_async_sleep::async_sleep;
/// use std::time::Duration;
/// #[cfg(not(target_arch = "wasm32"))]
/// use std::time::Instant;
/// #[cfg(target_arch = "wasm32")]
/// use web_time::Instant;
///
/// # use test_executors::async_test;
/// # #[async_test]
/// # async fn test() {
/// let duration = Duration::from_millis(100);
/// let start = Instant::now();
///
/// async_sleep(duration).await;
///
/// let elapsed = start.elapsed();
/// assert!(elapsed >= duration);
/// println!("Requested: {:?}, Actual: {:?}", duration, elapsed);
/// # }
/// ```
///
/// Concurrent sleeps:
///
/// ```
/// use portable_async_sleep::async_sleep;
/// use std::time::Duration;
/// #[cfg(not(target_arch = "wasm32"))]
/// use std::time::Instant;
/// #[cfg(target_arch = "wasm32")]
/// use web_time::Instant;
///
/// # use test_executors::async_test;
/// # #[async_test]
/// # async fn test() {
/// // Multiple concurrent sleeps complete in parallel, not sequentially
/// let start = Instant::now();
///
/// let futures = vec![
///     async_sleep(Duration::from_millis(100)),
///     async_sleep(Duration::from_millis(100)),
///     async_sleep(Duration::from_millis(100)),
/// ];
///
/// futures::future::join_all(futures).await;
///
/// // Total time should be ~100ms, not 300ms
/// let elapsed = start.elapsed();
/// assert!(elapsed < Duration::from_millis(150));
/// # }
/// ```
pub async fn async_sleep(duration: std::time::Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        stdlib::async_sleep(duration).await;
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm::async_sleep(duration).await;
    }
}

#[cfg(test)]
mod tests {
    use crate::async_sleep;
    #[cfg(not(target_arch = "wasm32"))]
    use std::time;
    #[cfg(target_arch = "wasm32")]
    use web_time as time;

    use test_executors::async_test;
    #[async_test]
    async fn test_async_sleep() {
        let duration = time::Duration::from_millis(500);
        let now = time::Instant::now();
        async_sleep(duration).await;
        let elapsed = now.elapsed();
        assert!(
            elapsed >= duration,
            "Expected at least 500ms, got {:?}",
            elapsed
        );
    }

    #[async_test]
    async fn test_join() {
        let duration = time::Duration::from_millis(500);
        let now = time::Instant::now();
        let f1 = async_sleep(duration);
        let f2 = async_sleep(duration);
        futures::join!(f1, f2);
        let elapsed = now.elapsed();
        assert!(elapsed >= duration);
        assert!(
            elapsed < std::time::Duration::from_millis(1000),
            "expected simultaneous sleep, got {:?}",
            elapsed
        );
    }

    #[test_executors::async_test]
    async fn test_join_2() {
        //require browser for wasm_thread
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        let start = time::Instant::now();

        // Create async blocks that each track their own completion time
        let f1 = async {
            let sleep_start = time::Instant::now();
            async_sleep(time::Duration::from_millis(150)).await;
            let elapsed = sleep_start.elapsed();
            ("f1", elapsed)
        };

        let f2 = async {
            let sleep_start = time::Instant::now();
            async_sleep(time::Duration::from_millis(50)).await;
            let elapsed = sleep_start.elapsed();
            ("f2", elapsed)
        };

        let f3 = async {
            let sleep_start = time::Instant::now();
            async_sleep(time::Duration::from_millis(100)).await;
            let elapsed = sleep_start.elapsed();
            ("f3", elapsed)
        };

        // Run all sleeps concurrently
        let (result1, result2, result3) = futures::join!(f1, f2, f3);

        let total_elapsed = start.elapsed();

        // Verify individual timing
        assert!(
            result1.1 >= time::Duration::from_millis(150),
            "{} completed too early: {:?}",
            result1.0,
            result1.1
        );
        assert!(
            result1.1 < time::Duration::from_millis(250),
            "{} took too long: {:?}",
            result1.0,
            result1.1
        );

        assert!(
            result2.1 >= time::Duration::from_millis(50),
            "{} completed too early: {:?}",
            result2.0,
            result2.1
        );
        assert!(
            result2.1 < time::Duration::from_millis(150),
            "{} took too long: {:?}",
            result2.0,
            result2.1
        );

        assert!(
            result3.1 >= time::Duration::from_millis(100),
            "{} completed too early: {:?}",
            result3.0,
            result3.1
        );
        assert!(
            result3.1 < time::Duration::from_millis(200),
            "{} took too long: {:?}",
            result3.0,
            result3.1
        );

        // Verify they ran concurrently (total time should be ~150ms, not 300ms)
        assert!(
            total_elapsed >= time::Duration::from_millis(150),
            "Total time too short: {:?}",
            total_elapsed
        );
        assert!(
            total_elapsed < time::Duration::from_millis(250),
            "Total time too long (not concurrent): {:?}",
            total_elapsed
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn test_future_is_send() {
        fn assert_send<T: Send>(_: T) {}

        let future = async_sleep(time::Duration::from_millis(100));
        assert_send(future);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_wasm_sleep() {
        let duration = std::time::Duration::from_millis(100);
        // Just test that it completes without error
        async_sleep(duration).await;
    }

    #[wasm_bindgen_test]
    async fn test_wasm_concurrent_sleeps() {
        let duration = std::time::Duration::from_millis(50);

        // Run multiple sleeps concurrently
        let sleep1 = async_sleep(duration);
        let sleep2 = async_sleep(duration);
        let sleep3 = async_sleep(duration);

        futures::join!(sleep1, sleep2, sleep3);
    }

    #[wasm_bindgen_test]
    async fn test_wasm_zero_duration() {
        // Test that zero duration sleep works
        async_sleep(std::time::Duration::from_millis(0)).await;
    }
}
