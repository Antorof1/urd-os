pub mod context;
pub mod scheduler;
pub mod thread;

pub use thread::{Thread, ThreadId};
use x86_64::VirtAddr;

use crate::thread::scheduler::SCHEDULER;

pub fn schedule() -> Option<(*mut VirtAddr, VirtAddr)> {
    SCHEDULER.try_lock().and_then(|mut s| s.schedule())
}
