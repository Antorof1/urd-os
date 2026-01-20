use core::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};

use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use futures_util::{Stream, StreamExt, task::AtomicWaker};
use spin::Mutex;

static TICKS: AtomicU64 = AtomicU64::new(0);
static WAKER: AtomicWaker = AtomicWaker::new();
static QUEUE: Mutex<BTreeMap<u64, Vec<Waker>>> = Mutex::new(BTreeMap::new());

pub fn on_timer_tick() {
    let _ = TICKS.fetch_add(1, Ordering::Release);

    WAKER.wake();
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
        let mut queue = QUEUE.lock();

        while let Some(entry) = queue.first_entry() {
            if *entry.key() > now {
                break;
            }

            let wakers = entry.remove();

            for waker in wakers {
                waker.wake();
            }
        }
    }
}
