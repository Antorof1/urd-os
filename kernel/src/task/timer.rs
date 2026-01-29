use core::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};

use alloc::collections::binary_heap::BinaryHeap;
use spin::Mutex;
use x86_64::instructions::interrupts;

static TICKS: AtomicU64 = AtomicU64::new(0);
static QUEUE: Mutex<BinaryHeap<TimerEntry>> = Mutex::new(BinaryHeap::new());

pub fn on_timer_tick() -> u64 {
    let now = TICKS.fetch_add(1, Ordering::Release) + 1;

    {
        let mut queue = QUEUE.lock();

        while let Some(entry) = queue.peek() {
            if entry.deadline > now {
                break;
            }

            let entry = queue.pop().unwrap();
            entry.waker.wake();
        }
    }

    now
}

pub struct Sleep {
    deadline: u64,
}

pub fn sleep(dur: Duration) -> Sleep {
    let current = TICKS.load(Ordering::Acquire);

    Sleep {
        deadline: current + dur.as_millis() as u64,
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let now = TICKS.load(Ordering::Acquire);

        if now >= self.deadline {
            return Poll::Ready(());
        }

        interrupts::without_interrupts(|| {
            QUEUE.lock().push(TimerEntry {
                deadline: self.deadline,
                waker: cx.waker().clone(),
            });
        });

        Poll::Pending
    }
}

struct TimerEntry {
    deadline: u64,
    waker: Waker,
}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}

impl Eq for TimerEntry {}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.deadline.cmp(&self.deadline)
    }
}
