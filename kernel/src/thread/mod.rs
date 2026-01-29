pub mod context;
pub mod scheduler;
pub mod thread;

pub use thread::{Thread, ThreadId};
use x86_64::instructions::interrupts;

use crate::thread::{context::switch_context, scheduler::scheduler};

pub fn schedule() {
    let stack_ptrs = scheduler().try_lock().and_then(|mut s| s.schedule());

    if let Some((current_stack_ptr, next_stack)) = stack_ptrs {
        unsafe {
            switch_context(current_stack_ptr, next_stack);
        }
    }
}

pub fn spawn(thread: Thread) {
    interrupts::without_interrupts(|| scheduler().lock().spawn(thread));
}

pub fn exit() -> ! {
    interrupts::without_interrupts(|| scheduler().lock().exit_current_thread())
}

pub fn yield_now() {
    interrupts::without_interrupts(|| scheduler().lock().block_current_thread());
}

pub fn wake(id: ThreadId) {
    interrupts::without_interrupts(|| scheduler().lock().wake_thread(id));
}

pub fn current_id() -> ThreadId {
    interrupts::without_interrupts(|| scheduler().lock().current_thread_id())
}
