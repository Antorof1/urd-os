use core::{
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use crossbeam_queue::ArrayQueue;

use crate::{
    task::{Task, TaskId, waker::ContextWaker},
    thread::{self, ThreadId},
};

static EXECUTOR_THREAD_ID: AtomicU64 = AtomicU64::new(0);

fn register_executor_thread() {
    let id = thread::current_id();

    if EXECUTOR_THREAD_ID
        .compare_exchange(0, id.as_u64(), Ordering::Release, Ordering::Relaxed)
        .is_err()
    {
        panic!("Failed to register executor: an executor thread is already registered")
    }
}

pub fn wake_executor_thread() {
    let id = EXECUTOR_THREAD_ID.load(Ordering::Acquire);

    if id == 0 {
        panic!("Failed to wake executor: no executor thread has been registered");
    }

    thread::wake(ThreadId::from_u64(id));
}

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
            panic!("Task NEXT_ID overflow");
        }

        self.task_queue
            .push(task_id)
            .expect("Executor queue is full");
    }

    pub fn run(&mut self) -> ! {
        register_executor_thread();

        loop {
            while let Some(task_id) = self.task_queue.pop() {
                let task = match self.tasks.get_mut(&task_id) {
                    Some(t) => t,
                    None => continue,
                };

                let waker_data =
                    Arc::new(ContextWaker::task(task_id, Arc::clone(&self.task_queue)));

                let waker = Waker::from(waker_data);
                let mut context = Context::from_waker(&waker);

                match task.poll(&mut context) {
                    Poll::Pending => {}
                    Poll::Ready(()) => {
                        self.tasks.remove(&task_id);
                    }
                }
            }

            thread::yield_now();
        }
    }
}
