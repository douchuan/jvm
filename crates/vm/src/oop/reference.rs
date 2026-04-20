use crate::oop::{ArrayOopDesc, InstOopDesc, MirrorOopDesc, TypeArrayDesc};
use std::fmt;
use std::fmt::Formatter;
use std::sync::{Condvar, Mutex};
use std::thread::ThreadId;
use std::time::Duration;

/// Reentrant mutex for Java monitor semantics.
/// Uses std::sync::{Mutex, Condvar} — no unsafe, no pthread.
pub struct Monitor {
    /// (owner_thread, recursion_count)
    state: Mutex<(Option<ThreadId>, usize)>,
    condvar: Condvar,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            state: Mutex::new((None, 0)),
            condvar: Condvar::new(),
        }
    }

    /// Acquire the monitor lock. Blocks if another thread owns it.
    pub fn lock(&self) {
        let current = std::thread::current().id();
        let mut guard = self.state.lock().unwrap();

        match guard.0 {
            Some(owner) if owner == current => {
                // Same thread re-entering
                guard.1 += 1;
            }
            Some(_) => {
                // Another thread owns it — wait
                loop {
                    guard = self.condvar.wait(guard).unwrap();
                    if guard.0.is_none() || guard.0 == Some(current) {
                        guard.0 = Some(current);
                        guard.1 = 1;
                        break;
                    }
                    // Spurious wakeup or still locked by another thread
                }
            }
            None => {
                // Uncontended acquisition
                guard.0 = Some(current);
                guard.1 = 1;
            }
        }
    }

    /// Release the monitor lock. Must be called by the owning thread.
    pub fn unlock(&self) {
        let current = std::thread::current().id();
        let mut guard = self.state.lock().unwrap();

        if guard.0 != Some(current) {
            panic!("unlock called by non-owner thread");
        }

        guard.1 -= 1;
        if guard.1 == 0 {
            guard.0 = None;
            self.condvar.notify_one();
        }
    }

    /// Wait on this monitor, releasing the lock. Re-acquires on wakeup.
    pub fn wait(&self) {
        // Must be called while holding the lock
        let current = std::thread::current().id();
        let mut guard = self.state.lock().unwrap();

        if guard.0 != Some(current) {
            panic!("wait called by non-owner thread");
        }

        // Release ownership and wait
        guard.0 = None;
        // Remember recursion count — we'll restore it on wakeup
        let saved_count = guard.1;
        guard.1 = 0;

        loop {
            guard = self.condvar.wait(guard).unwrap();
            if guard.0.is_none() {
                // Re-acquire with original recursion count
                guard.0 = Some(current);
                guard.1 = saved_count;
                break;
            }
        }
    }

    /// Wait with timeout. Returns true if notified, false if timed out.
    pub fn wait_timeout(&self, dur: Duration) -> bool {
        let current = std::thread::current().id();
        let mut guard = self.state.lock().unwrap();

        if guard.0 != Some(current) {
            panic!("wait_timeout called by non-owner thread");
        }

        let saved_count = guard.1;
        guard.0 = None;
        guard.1 = 0;

        let (new_guard, result) = self.condvar.wait_timeout(guard, dur).unwrap();
        guard = new_guard;

        if !result.timed_out() || guard.0.is_none() {
            guard.0 = Some(current);
            guard.1 = saved_count;
        } else {
            // Timed out — re-acquire the lock before returning
            drop(guard);
            self.lock();
        }

        !result.timed_out()
    }

    /// Wake up all threads waiting on this monitor.
    pub fn notify_all(&self) {
        self.condvar.notify_all();
    }

    pub fn notify_one(&self) {
        self.condvar.notify_one();
    }
}

/// Describes a heap-allocated object.
///
/// Contains the actual object data (`RefKind`) and monitor state for
/// `synchronized` blocks. Each `RefKindDesc` lives in a `Heap` slot.
pub struct RefKindDesc {
    pub v: RefKind,
    pub hash_code: Option<i32>,
    monitor: Monitor,
}

impl RefKindDesc {
    pub fn new(v: RefKind) -> Self {
        Self {
            v,
            hash_code: None,
            monitor: Monitor::new(),
        }
    }

    pub fn monitor_enter(&self) {
        self.monitor.lock();
    }

    pub fn monitor_exit(&self) {
        self.monitor.unlock();
    }

    pub fn wait(&self) {
        self.monitor.wait();
    }

    pub fn wait_timeout(&self, duration: Duration) {
        self.monitor.wait_timeout(duration);
    }

    pub fn notify_all(&self) {
        self.monitor.notify_all();
    }

    pub fn notify_one(&self) {
        self.monitor.notify_one();
    }
}

impl Drop for RefKindDesc {
    fn drop(&mut self) {
        // Monitor is dropped automatically via std::sync types
    }
}

impl fmt::Debug for RefKindDesc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RefKindDesc")
            .field("v", &self.v)
            .field("hash_code", &self.hash_code)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for RefKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RefKind::Inst(v) => f.debug_tuple("Inst").field(v).finish(),
            RefKind::Array(v) => f.debug_tuple("Array").field(v).finish(),
            RefKind::TypeArray(v) => f.debug_tuple("TypeArray").field(v).finish(),
            RefKind::Mirror(v) => f.debug_tuple("Mirror").field(v).finish(),
        }
    }
}

/// The kinds of heap-allocated objects in the JVM.
pub enum RefKind {
    Inst(InstOopDesc),       // Java object instance
    Array(ArrayOopDesc),     // Reference array (Object[], String[], etc.)
    TypeArray(TypeArrayDesc), // Primitive array (int[], byte[], char[], etc.)
    Mirror(MirrorOopDesc),   // java.lang.Class mirror
}

impl RefKind {
    pub fn extract_inst(&self) -> &InstOopDesc {
        match &self {
            RefKind::Inst(v) => v,
            _ => unreachable!("expected Inst, got {:?}", self),
        }
    }

    pub fn extract_array(&self) -> &ArrayOopDesc {
        match &self {
            RefKind::Array(v) => v,
            _ => unreachable!("expected Array, got {:?}", self),
        }
    }

    pub fn extract_mut_array(&mut self) -> &mut ArrayOopDesc {
        match self {
            RefKind::Array(v) => v,
            _ => unreachable!("expected Array, got {:?}", self),
        }
    }

    pub fn extract_type_array(&self) -> &TypeArrayDesc {
        match &self {
            RefKind::TypeArray(v) => v,
            _ => unreachable!("expected TypeArray, got {:?}", self),
        }
    }

    pub fn extract_mut_type_array(&mut self) -> &mut TypeArrayDesc {
        match self {
            RefKind::TypeArray(v) => v,
            _ => unreachable!("expected TypeArray, got {:?}", self),
        }
    }

    pub fn extract_mirror(&self) -> &MirrorOopDesc {
        match &self {
            RefKind::Mirror(v) => v,
            _ => unreachable!("expected Mirror, got {:?}", self),
        }
    }
}
