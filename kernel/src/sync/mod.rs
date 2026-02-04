use spin::Mutex;
use x86_64::instructions::interrupts;

#[derive(Debug)]
pub struct IrqLock<T> {
    inner: Mutex<T>,
}

impl<T> IrqLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            inner: Mutex::new(data),
        }
    }

    pub fn lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        interrupts::without_interrupts(|| {
            let mut guard = self.inner.lock();
            f(&mut *guard)
        })
    }

    pub fn try_lock<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        interrupts::without_interrupts(|| {
            if let Some(mut guard) = self.inner.try_lock() {
                Some(f(&mut *guard))
            } else {
                None
            }
        })
    }

    pub unsafe fn force_unlock(&self) {
        unsafe {
            self.inner.force_unlock();
        }
    }
}
