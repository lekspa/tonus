use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Timer {
    running: Arc<AtomicBool>,
    elapsed: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            elapsed: Arc::new(AtomicU64::new(0)),
            handle: None,
        }
    }

    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            return;
        }

        let running = Arc::clone(&self.running);
        let elapsed = Arc::clone(&self.elapsed);

        self.running.store(true, Ordering::Release);

        let handle = thread::spawn(move || {
            loop {
                // NOTE: Ordering of sleeping the thread and breaking the loop matters.
                // If changed, counter is incremented by 1s if playback is paused after 100ms of playing.
                // FIXME: Playing and pausing the track multiple times causes desynchronization between
                // actual playback time and timer elapsed time. Explore oportunity to sleep for n millis/nanos for higher presicion.
                thread::sleep(Duration::from_secs(1));

                if !running.load(Ordering::Acquire) {
                    break;
                }

                elapsed.fetch_add(1, Ordering::Relaxed);
            }
        });

        self.handle = Some(handle);
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Release);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn reset(&self) {
        self.elapsed.store(0, Ordering::Relaxed);
    }

    pub fn elapsed(&self) -> u64 {
        self.elapsed.load(Ordering::Relaxed)
    }

    pub fn set_elapsed(&self, value: u64) {
        self.elapsed.store(value, Ordering::Release);
    }
}
