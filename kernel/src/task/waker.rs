use alloc::{sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;

use crate::{
    task::{TaskId, executor::wake_executor_thread},
    thread::{self, ThreadId},
};

enum WakerType {
    TaskWaker {
        task_id: TaskId,
        task_queue: Arc<ArrayQueue<TaskId>>,
    },
    ThreadWaker {
        thread_id: ThreadId,
    },
}

pub struct ContextWaker {
    waker_type: WakerType,
}

impl ContextWaker {
    pub fn task(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Self {
        Self {
            waker_type: WakerType::TaskWaker {
                task_id,
                task_queue,
            },
        }
    }

    pub fn thread(thread_id: ThreadId) -> Self {
        Self {
            waker_type: WakerType::ThreadWaker { thread_id },
        }
    }
}

impl Wake for ContextWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        match &self.waker_type {
            WakerType::TaskWaker {
                task_id,
                task_queue,
            } => {
                task_queue
                    .push(task_id.clone())
                    .expect("Executor queue is full");

                wake_executor_thread();
            }

            WakerType::ThreadWaker { thread_id } => {
                thread::wake(thread_id.clone());
            }
        }
    }
}
