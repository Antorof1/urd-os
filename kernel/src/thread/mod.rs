pub mod context;
pub mod scheduler;
pub mod thread;

pub use thread::{Thread, ThreadId};

use crate::thread::{context::switch_context, scheduler::scheduler};

pub fn schedule() {
    let stack_ptrs = scheduler().try_lock(|s| s.schedule()).flatten();

    if let Some((current_stack_ptr, next_stack)) = stack_ptrs {
        unsafe {
            switch_context(current_stack_ptr, next_stack);
        }
    }
}

pub fn spawn(thread: Thread) {
    scheduler().lock(|s| s.spawn(thread));
}

pub fn exit() -> ! {
    scheduler().lock(|s| s.exit_current_thread())
}

pub fn yield_now() {
    scheduler().lock(|s| s.block_current_thread())
}

pub fn wake(id: ThreadId) {
    scheduler().lock(|s| s.wake_thread(id));
}

pub fn current_id() -> ThreadId {
    scheduler().lock(|s| s.current_thread_id())
}
