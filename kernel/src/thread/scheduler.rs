use alloc::collections::btree_map::BTreeMap;
use crossbeam_queue::ArrayQueue;
use spin::Mutex;
use x86_64::{VirtAddr, instructions::interrupts};

use crate::thread::{
    Thread, ThreadId,
    context::{switch_context, switch_to_context},
    thread::ThreadState,
};

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

pub struct Scheduler {
    threads: BTreeMap<ThreadId, Thread>,
    thread_queue: Option<ArrayQueue<ThreadId>>,
    dead_threads: Option<ArrayQueue<ThreadId>>,
    current_thread_id: Option<ThreadId>,
    idle_thread_id: Option<ThreadId>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            threads: BTreeMap::new(),
            thread_queue: None,
            dead_threads: None,
            current_thread_id: None,
            idle_thread_id: None,
        }
    }

    pub fn init(&mut self) {
        self.thread_queue = Some(ArrayQueue::new(1024));
        self.dead_threads = Some(ArrayQueue::new(1024));

        let idle_thread = Thread::new_idle();
        let idle_thread_id = idle_thread.id();

        self.threads.insert(idle_thread_id, idle_thread);

        self.idle_thread_id = Some(idle_thread_id);
    }

    pub fn run(&mut self) -> ! {
        let idle_id = self.idle_thread_id.expect("Scheduler called before init()");

        self.current_thread_id = Some(idle_id);

        let idle_thread = self.threads.get(&idle_id).unwrap();

        unsafe {
            SCHEDULER.force_unlock();

            switch_to_context(idle_thread.stack_ptr());
        }
        unreachable!()
    }

    pub fn spawn(&mut self, mut thread: Thread) {
        let thread_id = thread.id();

        thread.set_ready();

        if self.threads.insert(thread_id, thread).is_some() {
            panic!("Thread NEXT_ID overflow");
        }

        self.thread_queue_mut()
            .push(thread_id)
            .expect("Scheduler queue is full");
    }

    pub fn schedule(&mut self) -> Option<(*mut VirtAddr, VirtAddr)> {
        let dead_threads = self
            .dead_threads
            .as_mut()
            .expect("Scheduler called before init()");

        while let Some(id) = dead_threads.pop() {
            self.threads.remove(&id).unwrap();
        }

        let next_id = self.next_thread_id();

        let current_id = self
            .current_thread_id
            .expect("Scheduler called before run()");

        if current_id == next_id {
            return None;
        }

        unsafe {
            let threads_ptr = &mut self.threads as *mut BTreeMap<ThreadId, Thread>;

            let current_thread = (*threads_ptr).get_mut(&current_id).unwrap();
            let next_thread = (*threads_ptr).get_mut(&next_id).unwrap();

            if next_id == self.idle_thread_id.unwrap()
                && current_thread.state() == ThreadState::Running
            {
                return None;
            }

            if current_id != self.idle_thread_id.unwrap()
                && current_thread.state() == ThreadState::Running
            {
                self.thread_queue_mut()
                    .push(current_id)
                    .expect("Scheduler queue is full");
            }

            current_thread.set_ready();
            next_thread.set_running();

            self.current_thread_id = Some(next_id);

            Some((current_thread.stack_ptr_mut(), next_thread.stack_ptr()))
        }
    }

    pub fn exit_current_thread(&mut self) -> ! {
        interrupts::without_interrupts(|| {
            let current_id = self
                .current_thread_id
                .expect("Scheduler called before run()");

            if Some(current_id) == self.idle_thread_id {
                panic!("Cannot exit idle thread");
            }

            let current_thread = self.threads.get_mut(&current_id).unwrap();
            current_thread.set_dead();

            self.dead_threads
                .as_mut()
                .unwrap()
                .push(current_id)
                .expect("Dead threads queue is full");

            let next_id = self.next_thread_id();

            self.current_thread_id = Some(next_id);

            let next_thread = self.threads.get(&next_id).unwrap();

            unsafe {
                SCHEDULER.force_unlock();

                switch_to_context(next_thread.stack_ptr());
            }
        });
        unreachable!();
    }

    pub fn block_current_thread(&mut self) {
        interrupts::without_interrupts(|| {
            let current_id = self
                .current_thread_id
                .expect("Scheduler called before run()");

            if Some(current_id) == self.idle_thread_id {
                panic!("Cannot block idle thread");
            }

            let next_id = self.next_thread_id();

            unsafe {
                let threads_ptr = &mut self.threads as *mut BTreeMap<ThreadId, Thread>;

                let current_thread = (*threads_ptr).get_mut(&current_id).unwrap();
                let next_thread = (*threads_ptr).get_mut(&next_id).unwrap();

                current_thread.set_blocked();
                next_thread.set_running();

                self.current_thread_id = Some(next_id);

                SCHEDULER.force_unlock();

                switch_context(current_thread.stack_ptr_mut(), next_thread.stack_ptr());
            }
        });
    }

    fn next_thread_id(&mut self) -> ThreadId {
        while let Some(id) = self.thread_queue_mut().pop() {
            let thread = self.threads.get(&id).unwrap();

            if thread.state() == ThreadState::Ready {
                return id;
            }
        }

        self.idle_thread_id.unwrap()
    }

    fn thread_queue_mut(&mut self) -> &mut ArrayQueue<ThreadId> {
        self.thread_queue
            .as_mut()
            .expect("Scheduler called before init()")
    }
}
