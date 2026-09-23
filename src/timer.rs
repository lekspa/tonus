use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Timer {
    running: Arc<AtomicBool>,
    elapsed: Arc<AtomicU32>,
    handle: Option<JoinHandle<()>>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            elapsed: Arc::new(AtomicU32::new(0)),
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
                if !running.load(Ordering::Acquire) {
                    break;
                }

                thread::sleep(Duration::from_secs(1));
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

    pub fn elapsed(&self) -> u32 {
        self.elapsed.load(Ordering::Relaxed)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}
