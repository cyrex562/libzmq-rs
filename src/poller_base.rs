use std::collections::BTreeMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};

// Forward declaration of trait that must be implemented elsewhere
pub trait IPollEvents {
    fn timer_event(&mut self, id: i32);
}

struct TimerInfo {
    sink: Box<dyn IPollEvents>,
    id: i32,
}

pub struct Clock {
    // Implementation details would go here
}

impl Clock {
    pub fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

pub struct PollerBase {
    clock: Clock,
    timers: BTreeMap<u64, TimerInfo>,
    load: AtomicI32,
}

impl PollerBase {
    pub fn new() -> Self {
        PollerBase {
            clock: Clock {},
            timers: BTreeMap::new(),
            load: AtomicI32::new(0),
        }
    }

    pub fn get_load(&self) -> i32 {
        self.load.load(Ordering::Relaxed)
    }

    pub fn add_timer(&mut self, timeout: i32, sink: Box<dyn IPollEvents>, id: i32) {
        let expiration = self.clock.now_ms() + timeout as u64;
        let info = TimerInfo { sink, id };
        self.timers.insert(expiration, info);
    }

    pub fn cancel_timer(&mut self, sink_ptr: *const dyn IPollEvents, id: i32) {
        let to_remove: Vec<_> = self
            .timers
            .iter()
            .filter(|(_, info)| {
                std::ptr::eq(sink_ptr, &*info.sink as *const dyn IPollEvents) && info.id == id
            })
            .map(|(&k, _)| k)
            .collect();

        for key in to_remove {
            self.timers.remove(&key);
        }
    }

    fn adjust_load(&self, amount: i32) {
        if amount > 0 {
            self.load.fetch_add(amount, Ordering::Relaxed);
        } else if amount < 0 {
            self.load.fetch_sub(-amount, Ordering::Relaxed);
        }
    }

    fn execute_timers(&mut self) -> u64 {
        if self.timers.is_empty() {
            return 0;
        }

        let current = self.clock.now_ms();
        let mut expired = Vec::new();

        while let Some((&time, _)) = self.timers.first_key_value() {
            if time > current {
                return time - current;
            }

            if let Some((_, timer)) = self.timers.remove_entry(&time) {
                expired.push(timer);
            }
        }

        for timer in expired {
            timer.sink.timer_event(timer.id);
        }

        self.timers
            .first_key_value()
            .map(|(&time, _)| time - current)
            .unwrap_or(0)
    }
}

pub struct ThreadCtx {
    // Implementation details would go here
}

pub struct WorkerPollerBase {
    base: PollerBase,
    ctx: ThreadCtx,
    worker: Option<JoinHandle<()>>,
}

impl WorkerPollerBase {
    pub fn new(ctx: ThreadCtx) -> Self {
        WorkerPollerBase {
            base: PollerBase::new(),
            ctx,
            worker: None,
        }
    }

    pub fn start(&mut self, name: Option<&str>) {
        assert!(self.base.get_load() > 0);
        let worker = thread::Builder::new()
            .name(name.unwrap_or("worker").to_string())
            .spawn(move || {
                // Worker loop implementation would go here
            })
            .expect("Failed to spawn worker thread");

        self.worker = Some(worker);
    }

    pub fn stop_worker(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.join().expect("Worker thread panicked");
        }
    }

    #[cfg(debug_assertions)]
    fn check_thread(&self) {
        if let Some(worker) = &self.worker {
            assert!(thread::current().id() == worker.thread().id());
        }
    }
}

impl Drop for WorkerPollerBase {
    fn drop(&mut self) {
        assert_eq!(self.base.get_load(), 0);
    }
}
