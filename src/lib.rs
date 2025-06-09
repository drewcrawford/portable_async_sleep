/*!

![logo](../../../art/logo.png)

A portable async sleep function.

This function implements `async_sleep` in a totally runtime-agnostic way.


*/

mod stdlib;

/**
A portable async sleep function.

This function implements `async_sleep` in a totally runtime-agnostic way.
*/
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