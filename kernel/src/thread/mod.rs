pub mod context;
pub mod scheduler;
pub mod thread;

pub use thread::{Thread, ThreadId};

use crate::thread::{context::switch_context, scheduler::SCHEDULER};

pub fn schedule() {
    let stack_ptrs = SCHEDULER.try_lock().and_then(|mut s| s.schedule());

    if let Some((current_stack_ptr, next_stack)) = stack_ptrs {
        unsafe {
            switch_context(current_stack_ptr, next_stack);
        }
    }
}

pub fn exit() -> ! {
    SCHEDULER.lock().exit_current_thread();
}

pub fn yield_now() {
    SCHEDULER.lock().block_current_thread();
}
