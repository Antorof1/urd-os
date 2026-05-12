use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{sync::Arc, vec, vec::Vec};

use crate::{
    memory::vmm::vmm,
    process::{
        address_space::ProcessAddressSpace,
        thread::{Thread, ThreadId},
    },
    sync::IrqLock,
};

#[derive(Debug)]
pub struct Process {
    id: ProcessId,
    threads: IrqLock<Vec<Arc<Thread>>>,
    address_space: ProcessAddressSpace,
}

impl Process {
    pub fn new(thread: Arc<Thread>) -> Self {
        let address_space = vmm().lock(|vmm| ProcessAddressSpace::new(vmm).expect("Out of memory"));

        Self {
            id: ProcessId::new(),
            threads: IrqLock::new(vec![thread]),
            address_space,
        }
    }

    pub fn new_kernel(thread: Arc<Thread>) -> Self {
        Self {
            id: ProcessId::new(),
            threads: IrqLock::new(vec![thread]),
            address_space: ProcessAddressSpace::from_current(),
        }
    }

    pub fn add_thread(&self, thread: Arc<Thread>) {
        self.threads.lock(|t| t.push(thread));
    }

    pub fn add_new_thread(self: &Arc<Self>, entry: fn()) {
        let thread = Arc::new(Thread::new(Arc::downgrade(self), entry));

        self.threads.lock(|t| t.push(thread));
    }

    pub fn remove_thread(&self, id: ThreadId) -> bool {
        self.threads.lock(|threads| {
            if let Some(index) = threads.iter().position(|t| t.id() == id) {
                threads.swap_remove(index);
            }

            threads.is_empty()
        })
    }

    pub fn id(&self) -> ProcessId {
        self.id
    }

    pub fn cr3_value(&self) -> u64 {
        self.address_space.cr3_value()
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
