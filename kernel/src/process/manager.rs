use alloc::{collections::btree_map::BTreeMap, sync::Arc};

use crate::{
    process::{Process, ProcessId, thread},
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
}
