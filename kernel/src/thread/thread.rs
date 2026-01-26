use core::sync::atomic::{AtomicU64, Ordering};

use alloc::boxed::Box;
use x86_64::VirtAddr;

use crate::thread::{self, context::ContextFrame};

const STACK_SIZE: usize = 16384;

pub struct Thread {
    id: ThreadId,
    stack: Box<[u8; STACK_SIZE]>, // map to real frames instead of kernel heap
    stack_ptr: VirtAddr,
}

impl Thread {
    pub fn new(entry: fn()) -> Self {
        let mut stack = Box::new([0u8; STACK_SIZE]);

        unsafe {
            let stack_top = stack.as_mut_ptr().add(STACK_SIZE);

            let ret_addr_ptr = stack_top.sub(8);
            core::ptr::write(ret_addr_ptr as *mut u64, thread::exit as *const () as u64);

            let context_size = size_of::<ContextFrame>();
            let stack_ptr = ret_addr_ptr.sub(context_size);

            let context_ptr = stack_ptr as *mut ContextFrame;
            core::ptr::write_bytes(context_ptr, 0, 1);

            let context = &mut *context_ptr;

            context.rip = entry as u64;
            context.cs = 0x8;
            context.rflags = 0x202;
            context.rsp = ret_addr_ptr as u64;
            context.ss = 0x10;

            Self {
                id: ThreadId::new(),
                stack,
                stack_ptr: VirtAddr::from_ptr(stack_ptr),
            }
        }
    }

    pub fn new_idle() -> Self {
        Self::new(|| {
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
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ThreadId(u64);

impl ThreadId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
