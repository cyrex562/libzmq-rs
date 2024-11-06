use std::collections::{BTreeMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

type TimerCallback = Box<dyn Fn(i32, *mut libc::c_void)>;

#[derive(Clone)]
struct Timer {
    timer_id: i32,
    interval: usize,
    handler: TimerCallback,
    arg: *mut libc::c_void,
}

pub struct Timers {
    tag: u32,
    next_timer_id: i32,
    timers: BTreeMap<u64, Timer>,
    cancelled_timers: HashSet<i32>,
}

impl Timers {
    pub fn new() -> Self {
        Timers {
            tag: 0xCAFEDADA,
            next_timer_id: 0,
            timers: BTreeMap::new(),
            cancelled_timers: HashSet::new(),
        }
    }

    pub fn check_tag(&self) -> bool {
        self.tag == 0xCAFEDADA
    }

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    pub fn add(&mut self, interval: usize, handler: TimerCallback, arg: *mut libc::c_void) -> i32 {
        self.next_timer_id += 1;
        let timer = Timer {
            timer_id: self.next_timer_id,
            interval,
            handler,
            arg,
        };
        let when = Self::now_ms() + interval as u64;
        self.timers.insert(when, timer);
        self.next_timer_id
    }

    pub fn cancel(&mut self, timer_id: i32) -> Result<(), &'static str> {
        if !self.timers.values().any(|t| t.timer_id == timer_id) {
            return Err("Invalid timer ID");
        }

        if self.cancelled_timers.contains(&timer_id) {
            return Err("Timer already cancelled");
        }

        self.cancelled_timers.insert(timer_id);
        Ok(())
    }

    pub fn set_interval(&mut self, timer_id: i32, interval: usize) -> Result<(), &'static str> {
        let timer = self.timers
            .iter()
            .find(|(_, t)| t.timer_id == timer_id)
            .map(|(_, t)| t.clone());

        if let Some(mut timer) = timer {
            self.timers.retain(|_, t| t.timer_id != timer_id);
            timer.interval = interval;
            let when = Self::now_ms() + interval as u64;
            self.timers.insert(when, timer);
            Ok(())
        } else {
            Err("Invalid timer ID")
        }
    }

    pub fn reset(&mut self, timer_id: i32) -> Result<(), &'static str> {
        let timer = self.timers
            .iter()
            .find(|(_, t)| t.timer_id == timer_id)
            .map(|(_, t)| t.clone());

        if let Some(timer) = timer {
            self.timers.retain(|_, t| t.timer_id != timer_id);
            let when = Self::now_ms() + timer.interval as u64;
            self.timers.insert(when, timer);
            Ok(())
        } else {
            Err("Invalid timer ID")
        }
    }

    pub fn timeout(&mut self) -> i64 {
        let now = Self::now_ms();
        let mut result = -1i64;

        self.timers.retain(|&when, timer| {
            if self.cancelled_timers.remove(&timer.timer_id) {
                false
            } else {
                if result == -1 {
                    result = std::cmp::max(when as i64 - now as i64, 0);
                }
                true
            }
        });

        result
    }

    pub fn execute(&mut self) -> Result<(), &'static str> {
        let now = Self::now_ms();
        let mut new_timers = Vec::new();

        let expired_timers: Vec<_> = self.timers
            .range(..=now)
            .filter(|(_, timer)| !self.cancelled_timers.contains(&timer.timer_id))
            .map(|(_, timer)| timer.clone())
            .collect();

        self.timers.retain(|&when, timer| {
            when > now || self.cancelled_timers.contains(&timer.timer_id)
        });

        for timer in expired_timers {
            (timer.handler)(timer.timer_id, timer.arg);
            new_timers.push((now + timer.interval as u64, timer));
        }

        for (when, timer) in new_timers {
            self.timers.insert(when, timer);
        }

        Ok(())
    }
}

impl Drop for Timers {
    fn drop(&mut self) {
        self.tag = 0xdeadbeef;
    }
}
