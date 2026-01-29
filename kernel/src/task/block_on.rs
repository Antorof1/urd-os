use core::task::{Context, Poll, Waker};

use alloc::{boxed::Box, sync::Arc};

use crate::{task::waker::ContextWaker, thread};

pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);

    let thread_id = thread::current_id();

    let waker_data = Arc::new(ContextWaker::thread(thread_id));
    let waker = Waker::from(waker_data);
    let mut cx = Context::from_waker(&waker);

    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => thread::yield_now(),
        }
    }
}
