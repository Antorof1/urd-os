pub mod context;
pub mod manager;
pub mod process;
pub mod scheduler;
pub mod thread;

use core::time::Duration;

use alloc::sync::Arc;
pub use process::{Process, ProcessId};

use crate::{
    print,
    process::{
        context::switch_context, manager::PROCESS_MANAGER, scheduler::scheduler, thread::Thread,
    },
    task::{Task, executor::Executor, timer},
};

pub fn schedule() {
    let stack_ptrs = scheduler().try_lock(|s| s.schedule()).flatten();

    if let Some((current_stack_ptr, next_stack)) = stack_ptrs {
        unsafe {
            switch_context(current_stack_ptr, next_stack);
        }
    }
}

pub fn init() {
    let kernel_process = Arc::new_cyclic(|process| {
        let idle_thread = Thread::new_idle(process.clone());
        let idle_threada_arc = Arc::new(idle_thread);

        scheduler::init(Arc::clone(&idle_threada_arc));
        Process::new(idle_threada_arc)
    });

    PROCESS_MANAGER.lock(|pm| pm.spawn_process(kernel_process));
}

pub fn spawn(entry: fn()) {
    let process = Arc::new_cyclic(|process| {
        let thread = Thread::new(process.clone(), entry);
        let thread_arc = Arc::new(thread);

        Process::new(thread_arc)
    });

    PROCESS_MANAGER.lock(|pm| pm.spawn_process(process))
}

pub fn spawn_thread(entry: fn()) {
    let current_id = current_id();

    PROCESS_MANAGER.lock(|pm| pm.spawn_thread(current_id, entry));
}

pub fn current_id() -> ProcessId {
    scheduler().lock(|s| s.current_process_id())
}
