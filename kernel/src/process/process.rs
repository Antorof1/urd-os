use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{sync::Arc, vec, vec::Vec};

use crate::{process::thread::Thread, sync::IrqLock};

pub struct Process {
    id: ProcessId,
    threads: IrqLock<Vec<Arc<Thread>>>,
}

impl Process {
    pub fn new(thread: Arc<Thread>) -> Self {
        Self {
            id: ProcessId::new(),
            threads: IrqLock::new(vec![thread]),
        }
    }

    pub fn add_thread(self: &Arc<Self>, thread: Arc<Thread>) {
        self.threads.lock(|t| t.push(thread));
    }

    pub fn add_new_thread(self: &Arc<Self>, entry: fn()) {
        let thread = Arc::new(Thread::new(Arc::downgrade(self), entry));

        self.threads.lock(|t| t.push(thread));
    }

    pub fn id(&self) -> ProcessId {
        self.id
    }

    pub fn with_threads<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<Thread>]) -> R,
    {
        self.threads.lock(|threads| f(threads.as_slice()))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ProcessId(u64);

impl ProcessId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
