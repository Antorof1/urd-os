use core::{
    arch::naked_asm,
    sync::atomic::{AtomicU64, Ordering},
};

use alloc::boxed::Box;
use x86_64::{VirtAddr, instructions::interrupts};

use crate::thread::{self, context::Context};

const STACK_SIZE: usize = 16384;

pub struct Thread {
    id: ThreadId,
    state: ThreadState,
    stack: Box<[u8; STACK_SIZE]>, // map to real frames instead of kernel heap
    stack_ptr: VirtAddr,
}

impl Thread {
    pub fn new(entry: fn()) -> Self {
        let mut stack = Box::new([0u8; STACK_SIZE]);

        unsafe {
            let stack_top = stack.as_mut_ptr().add(STACK_SIZE);

            let context_size = size_of::<Context>();
            let stack_ptr = stack_top.sub(context_size);

            let context_ptr = stack_ptr as *mut Context;
            core::ptr::write_bytes(context_ptr, 0, 1);

            let context = &mut *context_ptr;
            context.rip = entry_wrapper as *const () as u64;
            context.r12 = entry as u64;

            Self {
                id: ThreadId::new(),
                state: ThreadState::New,
                stack,
                stack_ptr: VirtAddr::from_ptr(stack_ptr),
            }
        }
    }

    pub fn new_idle() -> Self {
        Self::new(|| {
            interrupts::enable();

            loop {
                x86_64::instructions::hlt();
            }
        })
    }

    pub fn id(&self) -> ThreadId {
        self.id
    }

    pub fn stack_ptr(&self) -> VirtAddr {
        self.stack_ptr
    }

    pub fn stack_ptr_mut(&mut self) -> &mut VirtAddr {
        &mut self.stack_ptr
    }

    pub fn state(&self) -> ThreadState {
        self.state
    }

    pub fn set_ready(&mut self) {
        self.state = ThreadState::Ready;
    }

    pub fn set_running(&mut self) {
        self.state = ThreadState::Running;
    }

    pub fn set_blocked(&mut self) {
        self.state = ThreadState::Blocked;
    }

    pub fn set_dead(&mut self) {
        self.state = ThreadState::Dead;
    }
}

#[unsafe(naked)]
extern "C" fn entry_wrapper() -> ! {
    naked_asm!("sti", "call r12", "call {thread_exit}", thread_exit = sym thread::exit);
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
