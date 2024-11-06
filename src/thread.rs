use std::thread::{self, JoinHandle};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashSet;

#[cfg(unix)]
use libc::{pthread_t, pthread_self, pthread_equal};

#[cfg(windows)]
use winapi::um::winnt::HANDLE;
#[cfg(windows)]
use winapi::um::processthreadsapi::GetCurrentThreadId;

pub type ThreadFn = Box<dyn FnOnce() + Send + 'static>;

pub struct Thread {
    name: [u8; 16],
    tfn: Option<ThreadFn>,
    started: bool,
    thread_priority: i32,
    thread_sched_policy: i32,
    thread_affinity_cpus: HashSet<i32>,
    handle: Option<JoinHandle<()>>,
    #[cfg(windows)]
    descriptor: Option<HANDLE>,
    #[cfg(windows)]
    thread_id: u32,
    #[cfg(unix)]
    descriptor: Option<pthread_t>,
}

const THREAD_PRIORITY_DEFAULT: i32 = -1;
const THREAD_SCHED_POLICY_DEFAULT: i32 = -1;

impl Thread {
    pub fn new() -> Self {
        Thread {
            name: [0; 16],
            tfn: None,
            started: false,
            thread_priority: THREAD_PRIORITY_DEFAULT,
            thread_sched_policy: THREAD_SCHED_POLICY_DEFAULT,
            thread_affinity_cpus: HashSet::new(),
            handle: None,
            #[cfg(windows)]
            descriptor: None,
            #[cfg(windows)]
            thread_id: 0,
            #[cfg(unix)]
            descriptor: None,
        }
    }

    pub fn start<F>(&mut self, func: F, name: Option<&str>) 
    where
        F: FnOnce() + Send + 'static
    {
        if let Some(name) = name {
            let bytes = name.as_bytes();
            let len = bytes.len().min(15);
            self.name[..len].copy_from_slice(&bytes[..len]);
        }

        let builder = thread::Builder::new();
        let builder = if !self.name.is_empty() {
            builder.name(String::from_utf8_lossy(&self.name).into_owned())
        } else {
            builder
        };

        let handle = builder.spawn(move || {
            #[cfg(unix)]
            Self::block_signals();
            
            func();
        }).expect("Failed to spawn thread");

        self.handle = Some(handle);
        self.started = true;
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn is_current_thread(&self) -> bool {
        #[cfg(windows)]
        {
            if let Some(handle) = &self.handle {
                unsafe { GetCurrentThreadId() == self.thread_id }
            } else {
                false
            }
        }
        #[cfg(unix)]
        {
            if let Some(descriptor) = self.descriptor {
                unsafe { pthread_equal(pthread_self(), descriptor) != 0 }
            } else {
                false
            }
        }
    }

    pub fn stop(&mut self) {
        if self.started {
            if let Some(handle) = self.handle.take() {
                handle.join().expect("Failed to join thread");
            }
            self.started = false;
        }
    }

    pub fn set_scheduling_parameters(
        &mut self,
        priority: i32,
        policy: i32,
        affinity_cpus: HashSet<i32>
    ) {
        self.thread_priority = priority;
        self.thread_sched_policy = policy;
        self.thread_affinity_cpus = affinity_cpus;
    }

    #[cfg(unix)]
    fn block_signals() {
        unsafe {
            let mut set: libc::sigset_t = std::mem::zeroed();
            libc::sigfillset(&mut set);
            libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
        }
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        self.stop();
    }
}
