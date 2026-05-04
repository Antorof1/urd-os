use alloc::{collections::btree_map::BTreeMap, sync::Arc};

use crate::{
    process::{
        Process, ProcessId,
        thread::{self, Thread},
    },
    sync::IrqLock,
};

pub static PROCESS_MANAGER: IrqLock<ProcessManager> = IrqLock::new(ProcessManager::new());

pub struct ProcessManager {
    processes: BTreeMap<ProcessId, Arc<Process>>,
}

impl ProcessManager {
    pub const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
        }
    }

    pub fn add_process(&mut self, process: Arc<Process>) {
        let id = process.id();

        if self.processes.insert(id, process).is_some() {
            panic!("Process NEXT_ID overflow");
        }
    }

    pub fn spawn_process(&mut self, process: Arc<Process>) {
        process.with_threads(|threads| {
            for thread in threads {
                thread::spawn(Arc::clone(thread));
            }
        });

        self.add_process(process);
    }

    pub fn spawn_thread<F>(&mut self, id: ProcessId, entry: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let process = self.processes.get(&id).expect("Process not found");

        let thread = Thread::new(Arc::downgrade(process), entry);
        let thread_arc = Arc::new(thread);

        thread::spawn(Arc::clone(&thread_arc));

        process.add_thread(thread_arc);
    }

    pub fn remove_process(&mut self, id: ProcessId) {
        self.processes.remove(&id).expect("Process not found");
    }
}
