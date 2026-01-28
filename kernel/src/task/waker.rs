use alloc::{sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;

use crate::task::{TaskId, executor::wake_executor_thread};

pub struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    pub fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Self {
        Self {
            task_id,
            task_queue,
        }
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.task_queue
            .push(self.task_id)
            .expect("Executor queue is full");

        wake_executor_thread();
    }
}
