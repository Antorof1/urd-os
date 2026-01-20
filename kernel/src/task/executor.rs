use core::task::{Context, Poll, Waker};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use crossbeam_queue::ArrayQueue;

use crate::task::{Task, TaskId, waker::TaskWaker};

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(128)),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;

        if self.tasks.insert(task_id, task).is_some() {
            panic!("NEXT_ID overflow");
        }

        self.task_queue
            .push(task_id)
            .expect("Executor queue is full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            while let Some(task_id) = self.task_queue.pop() {
                let task = match self.tasks.get_mut(&task_id) {
                    Some(t) => t,
                    None => continue,
                };

                let waker_data = Arc::new(TaskWaker::new(task_id, Arc::clone(&self.task_queue)));

                let waker = Waker::from(waker_data);
                let mut context = Context::from_waker(&waker);

                match task.poll(&mut context) {
                    Poll::Pending => {}
                    Poll::Ready(()) => {
                        self.tasks.remove(&task_id);
                    }
                }
            }

            use x86_64::instructions::interrupts;

            interrupts::disable();

            if self.task_queue.is_empty() {
                interrupts::enable_and_hlt();
            } else {
                interrupts::enable();
            }
        }
    }
}
