use std::sync::Mutex as StdMutex;

pub struct Mutex<T> {
    inner: StdMutex<T>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: StdMutex::new(value),
        }
    }

    pub fn lock(&self) -> std::sync::LockResult<std::sync::MutexGuard<'_, T>> {
        self.inner.lock()
    }

    pub fn try_lock(&self) -> std::sync::TryLockResult<std::sync::MutexGuard<'_, T>> {
        self.inner.try_lock()
    }
}

pub struct ScopedLock<'a, T> {
    _guard: std::sync::MutexGuard<'a, T>,
}

impl<'a, T> ScopedLock<'a, T> {
    pub fn new(mutex: &'a Mutex<T>) -> Self {
        Self {
            _guard: mutex.lock().unwrap(),
        }
    }
}

pub struct ScopedOptionalLock<'a, T> {
    _guard: Option<std::sync::MutexGuard<'a, T>>,
}

impl<'a, T> ScopedOptionalLock<'a, T> {
    pub fn new(mutex: Option<&'a Mutex<T>>) -> Self {
        Self {
            _guard: mutex.map(|m| m.lock().unwrap()),
        }
    }
}
