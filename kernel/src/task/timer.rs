use core::{
    pin::Pin,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};

use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use futures_util::{Stream, StreamExt, task::AtomicWaker};
use spin::Mutex;

static TICKS: AtomicU64 = AtomicU64::new(0);
static WAKER: AtomicWaker = AtomicWaker::new();
static QUEUE: Mutex<BTreeMap<u64, Vec<Waker>>> = Mutex::new(BTreeMap::new());
static HAS_SLEEPERS: AtomicBool = AtomicBool::new(false);

pub fn on_timer_tick() -> u64 {
    let current_tick = TICKS.fetch_add(1, Ordering::Release);

    if HAS_SLEEPERS.load(Ordering::Relaxed) {
        WAKER.wake();
    }

    current_tick
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

        QUEUE
            .lock()
            .entry(self.deadline)
            .or_insert_with(Vec::new)
            .push(cx.waker().clone());

        HAS_SLEEPERS.store(true, Ordering::Relaxed);

        Poll::Pending
    }
}

struct TickStream {
    last_tick: u64,
}

impl TickStream {
    pub fn new() -> Self {
        let last_tick = TICKS.load(Ordering::Acquire);

        Self { last_tick }
    }
}

impl Stream for TickStream {
    type Item = u64;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        WAKER.register(cx.waker());

        let now = TICKS.load(Ordering::Acquire);

        if now > self.last_tick {
            self.last_tick = now;
            return Poll::Ready(Some(now));
        }

        Poll::Pending
    }
}

pub async fn task() {
    let mut stream = TickStream::new();

    while let Some(now) = stream.next().await {
        if !HAS_SLEEPERS.load(Ordering::Relaxed) {
            continue;
        }

        let mut queue = QUEUE.lock();

        let pending = queue.split_off(&(now + 1));
        let expired = core::mem::replace(&mut *queue, pending);

        for (_, wakers) in expired {
            for waker in wakers {
                waker.wake();
            }
        }

        let is_empty = queue.is_empty();
        HAS_SLEEPERS.store(!is_empty, Ordering::Relaxed);
    }
}
