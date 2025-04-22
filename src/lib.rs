/*!
A portable async sleep function.

This function implements `async_sleep` in a totally runtime-agnostic way.


*/

mod stdlib;

pub async fn async_sleep(duration: std::time::Duration) {
    stdlib::async_sleep(duration).await;
}

#[cfg(test)]
mod tests {
    use test_executors::async_test;
    use crate::async_sleep;
    #[async_test] async fn test_async_sleep() {
        let duration = std::time::Duration::new(1, 0);
        let now = std::time::Instant::now();
        async_sleep(duration).await;
        let elapsed = now.elapsed();
        assert!(elapsed >= duration, "Expected at least 1 second, got {:?}", elapsed);
    }
}