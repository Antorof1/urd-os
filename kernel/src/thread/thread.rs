use core::{
    arch::naked_asm,
    sync::atomic::{AtomicU64, Ordering},
};

use x86_64::{
    VirtAddr,
    instructions::interrupts,
    structures::paging::{PageSize, Size4KiB},
};

use crate::thread::{self, context::Context, stack::Stack};

pub struct Thread {
    id: ThreadId,
    state: ThreadState,
    #[allow(dead_code)]
    stack: Stack,
    stack_ptr: VirtAddr,
}

impl Thread {
    pub fn new(entry: fn()) -> Self {
        const STACK_SIZE: u64 = 16 * 1024;
        const STACK_REGION_START: u64 = 0xFFFF_B000_0000_0000;

        let id = ThreadId::new();

        let stack_offset = id.as_u64() * (STACK_SIZE + Size4KiB::SIZE);
        let stack_start_address = VirtAddr::new(STACK_REGION_START + stack_offset);

        let stack =
            Stack::new(stack_start_address, STACK_SIZE).expect("Failed to allocate stack pages");

        unsafe {
            let stack_top = stack.top();

            let stack_ptr = stack_top - size_of::<Context>() as u64;

            let context_ptr: *mut Context = stack_ptr.as_mut_ptr();
            core::ptr::write_bytes(context_ptr, 0, 1);

            let context = &mut *context_ptr;
            context.rip = entry_wrapper as *const () as u64;
            context.r12 = entry as u64;

            Self {
                id,
                state: ThreadState::New,
                stack,
                stack_ptr: stack_ptr,
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
