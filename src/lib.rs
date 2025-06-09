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
use std::time::{Duration, Instant};

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

mod stdlib;

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
/// use std::time::{Duration, Instant};
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
/// use std::time::{Duration, Instant};
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
    stdlib::async_sleep(duration).await;
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::time::Duration;
    use test_executors::async_test;
    use crate::async_sleep;
    #[async_test] async fn test_async_sleep() {
        let duration = std::time::Duration::from_millis(500);
        let now = std::time::Instant::now();
        async_sleep(duration).await;
        let elapsed = now.elapsed();
        assert!(elapsed >= duration, "Expected at least 1 second, got {:?}", elapsed);
    }

    #[async_test] async fn test_join() {
        let duration = std::time::Duration::from_millis(500);
        let now = std::time::Instant::now();
        let f1 = async_sleep(duration);
        let f2 = async_sleep(duration);
        futures::join!(f1, f2);
        let elapsed = now.elapsed();
        assert!(elapsed >= duration);
        assert!(elapsed < std::time::Duration::from_millis(1000), "expected simultaneous sleep, got {:?}", elapsed);
    }

    #[test] fn test_join_2() {
        let mut f1 = async_sleep(Duration::from_millis(100));
        let mut f1 = unsafe{Pin::new_unchecked(&mut f1)};

        let mut f2 = async_sleep(Duration::from_millis(10));
        let mut f2 = unsafe{Pin::new_unchecked(&mut f2)};

        let mut f3 = async_sleep(Duration::from_millis(400));
        let mut f3 = unsafe{Pin::new_unchecked(&mut f3)};


        //kick off all 3
        _ = test_executors::poll_once(f1.as_mut());
        _ = test_executors::poll_once(f2.as_mut());
        _ = test_executors::poll_once(f3.as_mut());

        std::thread::sleep(Duration::from_millis(15));
        //poll f2 again
        assert!(test_executors::poll_once(f2.as_mut()).is_ready());
        assert!(test_executors::poll_once(f1.as_mut()).is_pending());
        assert!(test_executors::poll_once(f3.as_mut()).is_pending());

        std::thread::sleep(Duration::from_millis(90));
        //poll f1 again
        assert!(test_executors::poll_once(f1.as_mut()).is_ready());
        assert!(test_executors::poll_once(f3.as_mut()).is_pending());

        std::thread::sleep(Duration::from_millis(298));
        //poll f3 again
        assert!(test_executors::poll_once(f3.as_mut()).is_ready());
    }
}