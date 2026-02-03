pub mod stack;
pub mod thread;

pub use thread::{Thread, ThreadId, ThreadState};

use crate::process::scheduler::scheduler;

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
