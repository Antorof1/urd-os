pub mod context;
pub mod scheduler;
pub mod thread;

pub use thread::{Thread, ThreadId};

use crate::thread::{context::switch_context, scheduler::SCHEDULER};

pub fn schedule() {
    let context_ptrs = SCHEDULER.try_lock().and_then(|mut s| s.schedule());

    if let Some((current_context_ptr, next_context_ptr)) = context_ptrs {
        unsafe {
            switch_context(current_context_ptr, next_context_ptr);
        }
    }
}
