pub mod context;
pub mod scheduler;
pub mod thread;

use crate::process::{context::switch_context, scheduler::scheduler};

pub fn schedule() {
    let stack_ptrs = scheduler().try_lock(|s| s.schedule()).flatten();

    if let Some((current_stack_ptr, next_stack)) = stack_ptrs {
        unsafe {
            switch_context(current_stack_ptr, next_stack);
        }
    }
}
