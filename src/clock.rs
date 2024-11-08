use std::time;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

// use winapi::shared::minwindef::LARGE_INTEGER;
#[cfg(target_os = "windows")]
use winapi::{
    um::{
        // libloaderapi::{GetProcAddress, LoadLibraryA, FreeLibrary},
        sysinfoapi::{GetTickCount, GetTickCount64},
    },
    shared::ntdef::LARGE_INTEGER,
};

#[cfg(target_os = "macos")]
use mach::{
    clock::{clock_get_time, clock_serv_t},
    clock_types::*,
    mach_time::*,
};

const USECS_PER_MSEC: u64 = 1_000;
const NSECS_PER_USEC: u64 = 1_000;
const USECS_PER_SEC: u64 = 1_000_000;

pub struct Clock {
    last_tsc: u64,
    last_time: u64,
}

#[cfg(target_os = "macos")]
fn alt_clock_gettime(clock_id: clockid_t, ts: &mut timespec) -> i32 {
    unsafe {
        let mut cclock: clock_serv_t = std::mem::zeroed();
        let mut mts: mach_timespec_t = std::mem::zeroed();
        
        host_get_clock_service(mach_host_self(), clock_id, &mut cclock);
        clock_get_time(cclock, &mut mts);
        mach_port_deallocate(mach_task_self(), cclock);
        
        ts.tv_sec = mts.tv_sec;
        ts.tv_nsec = mts.tv_nsec;
        0
    }
}

impl Clock {
    pub fn new() -> Self {
        // let now = Self::rdtsc();
        let now = time::SystemTime::now();
        let now_ms = now.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let now_ns = now.duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        Clock {
            last_tsc: now_ns,
            last_time: Self::now_ms(),
        }
    }

    pub fn now_us() -> u64 {
        #[cfg(target_os = "windows")]
        {
            // let mut ticks_per_second: LARGE_INTEGER = unsafe { std::mem::zeroed() };
            // let mut tick: LARGE_INTEGER = unsafe { std::mem::zeroed() };
            // unsafe {
            //     winapi::um::profileapi::QueryPerformanceFrequency(&mut ticks_per_second);
            //     winapi::um::profileapi::QueryPerformanceCounter(&mut tick);
            //     
            //     let ticks_div = (ticks_per_second.QuadPart() as f64) / (USECS_PER_SEC as f64);
            //     (tick.QuadPart() as f64 / ticks_div) as u64
            // }
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            ts.as_micros() as u64
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            ts.as_micros() as u64
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            ts.as_micros() as u64
        }
    }

    pub fn now_ms() -> u64 {
        #[cfg(target_os = "windows")]
        unsafe {
            GetTickCount64() as u64
        }

        #[cfg(not(target_os = "windows"))]
        {
            Self::now_us() / USECS_PER_MSEC
        }
    }

    // pub fn rdtsc() -> u64 {
    //     #[cfg(target_arch = "x86_64")]
    //     unsafe {
    //         #[cfg(target_os = "windows")]
    //         {
    //             core::arch::x86::_rdtsc()
    //         }
    //         #[cfg(not(target_os = "windows"))]
    //         {
    //             let mut low: u32;
    //             let mut high: u32;
    //             std::arch::asm!("rdtsc", out("eax") low, out("edx") high);
    //             ((high as u64) << 32) | (low as u64)
    //         }
    //     }
    // 
    //     #[cfg(target_arch = "aarch64")]
    //     unsafe {
    //         let mut pmccntr: u64;
    //         std::arch::asm!("mrs {}, pmccntr_el0", out(reg) pmccntr);
    //         pmccntr
    //     }
    // 
    //     #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    //     {
    //         // Fallback for other architectures
    //         let ts = SystemTime::now()
    //             .duration_since(UNIX_EPOCH)
    //             .unwrap_or_default();
    //         ts.as_nanos() as u64
    //     }
    // }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}