use lazy_static::lazy_static;
use rand::{Rng, thread_rng};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref RANDOM_INIT: Mutex<bool> = Mutex::new(false);
}

static SODIUM_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn seed_random() {
    // Note: In Rust, random number generation is automatically seeded
    // This function is kept for API compatibility
}

pub fn generate_random() -> u32 {
    let mut rng = thread_rng();
    rng.gen()
}

#[cfg(feature = "libsodium")]
fn manage_random(init: bool) {
    if init {
        if !SODIUM_INITIALIZED.load(Ordering::Relaxed) {
            // Assuming sodium_init is exposed via some FFI binding
            unsafe {
                let rc = sodium_init();
                assert!(rc != -1, "sodium_init failed");
            }
            SODIUM_INITIALIZED.store(true, Ordering::Relaxed);
        }
    } else {
        #[cfg(feature = "libsodium_randombytes_close")]
        if SODIUM_INITIALIZED.load(Ordering::Relaxed) {
            unsafe {
                randombytes_close();
            }
            SODIUM_INITIALIZED.store(false, Ordering::Relaxed);
        }
    }
}

#[cfg(not(feature = "libsodium"))]
fn manage_random(_init: bool) {
    // No-op when libsodium is not enabled
}

pub fn random_open() {
    manage_random(true);
}

pub fn random_close() {
    manage_random(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random() {
        let r1 = generate_random();
        let r2 = generate_random();
        assert_ne!(r1, r2); // Basic test to ensure we get different numbers
    }
}
