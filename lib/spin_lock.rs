use std::sync::atomic::AtomicBool;

pub struct SpinLock {
    flag: AtomicBool
}

impl SpinLock {
    pub fn new() -> Self {
        Self{flag: AtomicBool::new(false)}
    }

    pub fn lock(&self) {
        while self.flag.swap(true, std::sync::atomic::Ordering::Acquire) {}
    }

    pub fn release(&self) {
        self.flag.store(false,std::sync::atomic::Ordering::Release);
    }
}

pub struct SpinLockGuard<'a> {
    lock: &'a SpinLock
}

impl<'a> SpinLockGuard<'a> {
    pub fn new(lock: &'a SpinLock) -> Self {
        lock.lock();
        Self { lock }
    }
}

impl<'a> Drop for SpinLockGuard<'a> {
    fn drop(&mut self) {
        self.lock.release();
    }
}

impl SpinLock {
    pub fn make_guard(&'_ self)-> SpinLockGuard<'_> {
        SpinLockGuard::new(self)
    }
}

// TODO tests
