// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Implementation based on standard-library primitives.

There may be faster implementations, but this is the most portable one.
*/

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

// Keeping each platform timer reasonably short avoids `Instant` overflow. A
// very long sleep is transparently rescheduled in chunks.
const MAX_TIMER_SLICE: Duration = Duration::from_secs(24 * 60 * 60);

type TimerId = usize;
type TimerKey = (Instant, TimerId);

struct Timer {
    remaining: Duration,
    continuation: Option<r#continue::Sender<()>>,
    cancellation: Arc<AtomicBool>,
}

impl Timer {
    fn complete(mut self) {
        self.send();
    }

    fn send(&mut self) {
        if let Some(continuation) = self.continuation.take() {
            continuation.send(());
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // A continue::Sender panics if it is dropped without sending. Sending
        // to an already-cancelled continuation is explicitly a no-op.
        self.send();
    }
}

enum Request {
    Schedule {
        started: Instant,
        duration: Duration,
        continuation: r#continue::Sender<()>,
        cancellation: Arc<AtomicBool>,
    },
    Cancel(Arc<AtomicBool>),
    #[cfg(test)]
    IsScheduled {
        id: TimerId,
        response: mpsc::SyncSender<bool>,
    },
}

struct Cancellation {
    scheduler: Sender<Request>,
    cancellation: Arc<AtomicBool>,
}

impl r#continue::FutureCancellation for Cancellation {
    fn cancel(&mut self) {
        self.cancellation.store(true, Ordering::Release);

        // The scheduler may only be unavailable during process teardown or
        // after an unrecoverable worker failure. Cancellation must not panic in
        // either case.
        let _ = self
            .scheduler
            .send(Request::Cancel(Arc::clone(&self.cancellation)));
    }
}

fn timer_id(cancellation: &Arc<AtomicBool>) -> TimerId {
    Arc::as_ptr(cancellation) as TimerId
}

fn next_slice(now: Instant, remaining: Duration) -> (Instant, Duration) {
    let mut slice = remaining.min(MAX_TIMER_SLICE);

    loop {
        if let Some(deadline) = now.checked_add(slice) {
            return (deadline, remaining - slice);
        }

        // This is only relevant on targets with an unusually narrow Instant
        // range. Zero is always representable, so this loop terminates.
        slice /= 2;
    }
}

fn insert_timer(
    timers: &mut BTreeMap<TimerKey, Timer>,
    deadlines: &mut HashMap<TimerId, Instant>,
    now: Instant,
    mut timer: Timer,
) {
    let id = timer_id(&timer.cancellation);
    let (deadline, remaining) = next_slice(now, timer.remaining);
    timer.remaining = remaining;

    debug_assert!(deadlines.insert(id, deadline).is_none());
    debug_assert!(timers.insert((deadline, id), timer).is_none());
}

fn handle_request(
    request: Request,
    timers: &mut BTreeMap<TimerKey, Timer>,
    deadlines: &mut HashMap<TimerId, Instant>,
) {
    match request {
        Request::Schedule {
            started,
            duration,
            continuation,
            cancellation,
        } => {
            let elapsed = Instant::now().saturating_duration_since(started);
            let remaining = duration.saturating_sub(elapsed);
            let timer = Timer {
                remaining,
                continuation: Some(continuation),
                cancellation,
            };

            if timer.cancellation.load(Ordering::Acquire) || remaining.is_zero() {
                timer.complete();
            } else {
                insert_timer(timers, deadlines, Instant::now(), timer);
            }
        }
        Request::Cancel(cancellation) => {
            let id = timer_id(&cancellation);
            if let Some(deadline) = deadlines.remove(&id) {
                if let Some(timer) = timers.remove(&(deadline, id)) {
                    timer.complete();
                }
            }
        }
        #[cfg(test)]
        Request::IsScheduled { id, response } => {
            let _ = response.send(deadlines.contains_key(&id));
        }
    }
}

fn run_scheduler(receiver: Receiver<Request>) {
    let mut timers: BTreeMap<TimerKey, Timer> = BTreeMap::new();
    let mut deadlines: HashMap<TimerId, Instant> = HashMap::new();

    loop {
        let now = Instant::now();

        while let Some((&(deadline, id), _)) = timers.first_key_value() {
            if deadline > now {
                break;
            }

            let (_, timer) = timers
                .pop_first()
                .expect("the first timer disappeared while processing it");
            deadlines.remove(&id);

            if timer.cancellation.load(Ordering::Acquire) || timer.remaining.is_zero() {
                timer.complete();
            } else {
                insert_timer(&mut timers, &mut deadlines, Instant::now(), timer);
            }
        }

        let request = if let Some((&(deadline, _), _)) = timers.first_key_value() {
            match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Ok(request) => Some(request),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        } else {
            match receiver.recv() {
                Ok(request) => Some(request),
                Err(mpsc::RecvError) => break,
            }
        };

        if let Some(request) = request {
            handle_request(request, &mut timers, &mut deadlines);
        }
    }
}

fn start_scheduler() -> Sender<Request> {
    let (sender, receiver) = mpsc::channel();
    std::thread::Builder::new()
        .name("portable-async-sleep".to_owned())
        .spawn(move || run_scheduler(receiver))
        .expect("failed to spawn the async sleep scheduler");
    sender
}

static SCHEDULER: LazyLock<Sender<Request>> = LazyLock::new(start_scheduler);

pub async fn async_sleep(duration: Duration) {
    if duration.is_zero() {
        return;
    }

    let cancellation = Arc::new(AtomicBool::new(false));
    let cancel_handler = Cancellation {
        scheduler: SCHEDULER.clone(),
        cancellation: Arc::clone(&cancellation),
    };
    let (continuation, receiver) = r#continue::continuation_cancel(cancel_handler);

    SCHEDULER
        .send(Request::Schedule {
            started: Instant::now(),
            duration,
            continuation,
            cancellation,
        })
        .expect("the async sleep scheduler stopped unexpectedly");

    receiver.await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_scheduled(scheduler: &Sender<Request>, id: TimerId) -> bool {
        let (response, receiver) = mpsc::sync_channel(0);
        scheduler
            .send(Request::IsScheduled { id, response })
            .unwrap();
        receiver.recv().unwrap()
    }

    #[test]
    fn cancellation_removes_long_timer() {
        let scheduler = start_scheduler();
        let cancellation = Arc::new(AtomicBool::new(false));
        let id = timer_id(&cancellation);
        let cancel_handler = Cancellation {
            scheduler: scheduler.clone(),
            cancellation: Arc::clone(&cancellation),
        };
        let (continuation, future) = r#continue::continuation_cancel(cancel_handler);

        scheduler
            .send(Request::Schedule {
                started: Instant::now(),
                duration: Duration::MAX,
                continuation,
                cancellation,
            })
            .unwrap();
        assert!(is_scheduled(&scheduler, id));

        drop(future);
        assert!(!is_scheduled(&scheduler, id));
    }
}
