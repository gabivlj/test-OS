use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TaskId(u64);

use core::sync::atomic::{AtomicU64, Ordering};

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    pub(crate) id: TaskId,
    // Pin doesn't let access to deref mut (so it's safe to have self references)
    future: Pin<Box<dyn Future<Output = ()>>>,
}

use alloc::sync::Arc;
use core::cell::RefCell;
use core::task::{Context, Poll};

impl Task {
    // pub fn from_raw_(referen: usize) -> Self {
    //     let reference: *mut core::raw::TraitObject = unsafe { core::mem::transmute(referen) };
    //     let s: &dyn Future<Output = ()> = unsafe { core::mem::transmute(*reference) };
    //     let z: *mut dyn Future<Output = ()> = unsafe { core::mem::transmute(s) };
    //     let fu: Box<dyn Future<Output = ()>> = unsafe { alloc::boxed::Box::from_raw(z) };
    //     Self {
    //         id: TaskId::new(),
    //         future: Box::into_pin(fu),
    //     }
    // }

    pub fn from_raw(reference: usize) -> Arc<RefCell<Task>> {
        let reference: Arc<RefCell<Task>> = unsafe { core::mem::transmute(reference) };
        reference
    }

    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    pub fn from(future: Pin<Box<dyn Future<Output = ()>>>) -> Self {
        Self {
            id: TaskId::new(),
            future,
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
