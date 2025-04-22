/*!
Implementation based on stdlib primitives.

There may be faster implementations, but this is the most portable one.
*/

use std::sync::mpsc::Sender;
use std::time::Instant;

struct Message {
    instant: std::time::Instant,
    continuation: Option<r#continue::Sender<()>>,
}

impl Drop for Message {
    fn drop(&mut self) {
        // If the message is dropped, we need to wake up the thread
        // that is waiting for the message to be sent.
        self.continuation.take().unwrap().send(())
    }
}


const CHANNEL: std::sync::LazyLock<Sender<Message>> = std::sync::LazyLock::new(|| {
    let (sender, receiver) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let mut messages: Vec<Message> = Vec::new();
        loop {
            let before_wait_now = Instant::now();
            //calculate our timeout
            let timeout = if let Some(next) = messages.first() {
                next.instant.saturating_duration_since(before_wait_now)
            } else {
                std::time::Duration::from_secs(1000)
            };
            println!("Waiting for {}ms", timeout.as_millis());
            match receiver.recv_timeout(timeout) {
                Ok(message) => {
                    println!("Received message");
                    messages.push(message);
                    messages.sort_by(|a, b| a.instant.cmp(&b.instant));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    println!("Timed out");
                    let now = Instant::now();
                    messages.retain(|e| e.instant > now);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    println!("Disconnected");
                    break
                },
            }

        }
    });
    sender
});


pub async fn async_sleep(duration: std::time::Duration) {
    let future_instant = std::time::Instant::now() + duration;
    let (cs,cr) = r#continue::continuation();
    let message = Message {
        instant: future_instant,
        continuation: Some(cs),
    };
    CHANNEL.send(message).unwrap();
    cr.await
}