use core::{
    arch::naked_asm,
    cell::UnsafeCell,
    sync::atomic::{AtomicU64, Ordering},
};

use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
};
use x86_64::{
    VirtAddr,
    instructions::interrupts,
    structures::paging::{PageSize, Size4KiB},
};

use crate::{
    process::{
        Process,
        context::Context,
        thread::{self, stack::Stack},
    },
    sync::IrqLock,
};

#[derive(Debug)]
pub struct Thread {
    id: ThreadId,
    state: IrqLock<ThreadState>,
    parent_process: Weak<Process>,
    #[allow(dead_code)]
    stack: Stack,
    stack_ptr: UnsafeCell<VirtAddr>,
}

impl Thread {
    pub fn new<F>(parent_process: Weak<Process>, entry: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        const STACK_SIZE: u64 = 16 * 1024;
        const STACK_REGION_START: u64 = 0xFFFF_B000_0000_0000;

        let id = ThreadId::new();

        let stack_offset = id.as_u64() * (STACK_SIZE + Size4KiB::SIZE);
        let stack_start_address = VirtAddr::new(STACK_REGION_START + stack_offset);

        let stack =
            Stack::new(stack_start_address, STACK_SIZE).expect("Failed to allocate stack pages");

        let closure = Box::into_raw(Box::new(entry));

        unsafe {
            let stack_top = stack.top();

            let stack_ptr = stack_top - size_of::<Context>() as u64;

            let context_ptr: *mut Context = stack_ptr.as_mut_ptr();
            core::ptr::write_bytes(context_ptr, 0, 1);

            let context = &mut *context_ptr;
            context.rip = entry_wrapper as *const () as u64;
            context.r12 = thread_trampoline::<F> as *const () as u64;
            context.r13 = closure as u64;

            Self {
                id,
                state: IrqLock::new(ThreadState::New),
                parent_process,
                stack,
                stack_ptr: UnsafeCell::new(stack_ptr),
            }
        }
    }

    pub fn new_idle(parent_process: Weak<Process>) -> Self {
        Self::new(parent_process, || {
            interrupts::enable();

            loop {
                x86_64::instructions::hlt();
            }
        })
    }

    pub fn id(&self) -> ThreadId {
        self.id
    }

    pub fn parent_process(&self) -> Option<Arc<Process>> {
        self.parent_process.upgrade()
    }

    pub fn stack_ptr(&self) -> VirtAddr {
        unsafe { *self.stack_ptr.get() }
    }

    pub fn stack_ptr_mut(&self) -> &mut VirtAddr {
        unsafe { self.stack_ptr.as_mut_unchecked() }
    }

    pub fn state(&self) -> ThreadState {
        self.state.lock(|state| state.clone())
    }

    pub fn set_ready(&self) {
        self.state.lock(|state| *state = ThreadState::Ready);
    }

    pub fn set_running(&self) {
        self.state.lock(|state| *state = ThreadState::Running);
    }

    pub fn set_blocked(&self) {
        self.state.lock(|state| *state = ThreadState::Blocked);
    }

    pub fn set_dead(&self) {
        self.state.lock(|state| *state = ThreadState::Dead);
    }
}

unsafe impl Sync for Thread {}

extern "C" fn thread_trampoline<F>(closure_ptr: u64)
where
    F: FnOnce() + Send + 'static,
{
    let closure = unsafe { Box::from_raw(closure_ptr as *mut F) };

    closure();
}

#[unsafe(naked)]
extern "C" fn entry_wrapper() -> ! {
    naked_asm!("sti", "mov rdi, r13", "call r12", "call {thread_exit}", thread_exit = sym thread::exit);
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ThreadId(u64);

impl ThreadId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn from_u64(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ThreadState {
    New,
    Ready,
    Running,
    Blocked,
    Dead,
}
