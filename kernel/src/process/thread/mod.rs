pub mod stack;
pub mod thread;

use alloc::sync::Arc;
pub use thread::{Thread, ThreadId, ThreadState};

use crate::process::{manager::PROCESS_MANAGER, scheduler::scheduler};

pub fn spawn(thread: Arc<Thread>) {
    scheduler().lock(|s| s.spawn(thread));
}

pub fn exit() -> ! {
    let current_thread = scheduler().lock(|s| s.current_thread());

    if let Some(process) = current_thread.parent_process() {
        if process.remove_thread(current_thread.id()) {
            PROCESS_MANAGER.lock(|pm| pm.remove_process(process.id()));
        }
    }

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
