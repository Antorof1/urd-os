use alloc::collections::btree_map::BTreeMap;
use crossbeam_queue::ArrayQueue;
use spin::{Mutex, Once};
use x86_64::VirtAddr;

use crate::thread::{
    Thread, ThreadId,
    context::{switch_context, switch_to_context},
    thread::ThreadState,
};

static SCHEDULER: Once<Mutex<Scheduler>> = Once::new();

pub(super) fn scheduler() -> &'static Mutex<Scheduler> {
    SCHEDULER.get().expect("Scheduler called before init()")
}

pub fn init() {
    SCHEDULER.call_once(|| Mutex::new(Scheduler::new()));
}

pub fn run() -> ! {
    scheduler().lock().run()
}

pub struct Scheduler {
    threads: BTreeMap<ThreadId, Thread>,
    thread_queue: ArrayQueue<ThreadId>,
    dead_threads: ArrayQueue<ThreadId>,
    current_thread_id: Option<ThreadId>,
    idle_thread_id: ThreadId,
}

impl Scheduler {
    pub fn new() -> Self {
        let idle_thread = Thread::new_idle();
        let idle_thread_id = idle_thread.id();

        let mut threads = BTreeMap::new();

        threads.insert(idle_thread_id, idle_thread);

        Self {
            threads,
            thread_queue: ArrayQueue::new(1024),
            dead_threads: ArrayQueue::new(1024),
            current_thread_id: None,
            idle_thread_id: idle_thread_id,
        }
    }

    pub fn run(&mut self) -> ! {
        self.current_thread_id = Some(self.idle_thread_id);

        let idle_thread = self
            .threads
            .get(&self.idle_thread_id)
            .expect("Inserted in new()");

        unsafe {
            scheduler().force_unlock();

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

        self.thread_queue
            .push(thread_id)
            .expect("Scheduler queue is full");
    }

    pub fn schedule(&mut self) -> Option<(*mut VirtAddr, VirtAddr)> {
        while let Some(id) = self.dead_threads.pop() {
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

            if next_id == self.idle_thread_id && current_thread.state() == ThreadState::Running {
                return None;
            }

            if current_id != self.idle_thread_id && current_thread.state() == ThreadState::Running {
                self.thread_queue
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
        let current_id = self
            .current_thread_id
            .expect("Scheduler called before run()");

        if current_id == self.idle_thread_id {
            panic!("Cannot exit idle thread");
        }

        let current_thread = self.threads.get_mut(&current_id).unwrap();
        current_thread.set_dead();

        self.dead_threads
            .push(current_id)
            .expect("Dead threads queue is full");

        let next_id = self.next_thread_id();

        self.current_thread_id = Some(next_id);

        let next_thread = self.threads.get(&next_id).unwrap();

        unsafe {
            scheduler().force_unlock();

            switch_to_context(next_thread.stack_ptr());
        }
        unreachable!();
    }

    pub fn block_current_thread(&mut self) {
        let current_id = self
            .current_thread_id
            .expect("Scheduler called before run()");

        if current_id == self.idle_thread_id {
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

            scheduler().force_unlock();

            switch_context(current_thread.stack_ptr_mut(), next_thread.stack_ptr());
        }
    }

    pub fn wake_thread(&mut self, id: ThreadId) {
        let thread = self.threads.get_mut(&id).expect("Thread not found");

        match thread.state() {
            ThreadState::Ready => return,
            ThreadState::Running => {}
            _ => thread.set_ready(),
        }

        self.thread_queue.push(id).expect("Scheduler queue is full");
    }

    pub fn current_thread_id(&self) -> ThreadId {
        self.current_thread_id
            .expect("Scheduler called before run()")
    }

    fn next_thread_id(&mut self) -> ThreadId {
        while let Some(id) = self.thread_queue.pop() {
            let thread = self.threads.get(&id).unwrap();

            match thread.state() {
                ThreadState::Ready | ThreadState::Running => return id,
                _ => continue,
            }
        }

        self.idle_thread_id
    }
}
