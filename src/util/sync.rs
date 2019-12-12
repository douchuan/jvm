#![allow(unused)]

use std::ops::DerefMut;
use std::sync::{Mutex, MutexGuard};

pub fn sync_call<F, R, T>(mutex: &Mutex<T>, f: F) -> R
where
    F: FnOnce(MutexGuard<T>) -> R,
{
    let lock = mutex.lock().unwrap();
    f(lock)
}

pub fn sync_call_ctx<F, T, R>(mutex: &Mutex<T>, f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    let mut lock = mutex.lock().unwrap();
    let obj: &mut T = lock.deref_mut();
    f(obj)
}
