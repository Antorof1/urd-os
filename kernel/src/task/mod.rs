use core::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

use alloc::boxed::Box;

pub mod block_on;
pub mod executor;
pub mod timer;
pub mod waker;

pub use block_on::block_on;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    id: TaskId,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            future: Box::pin(future),
            id: TaskId::new(),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
