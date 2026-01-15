use core::alloc::{GlobalAlloc, Layout};

use spin::Mutex;
use talc::{ErrOnOom, Span, Talc, Talck};
use x86_64::{VirtAddr, instructions::interrupts};

#[global_allocator]
pub static ALLOCATOR: LockedAllocator = LockedAllocator::new();

pub struct LockedAllocator(Talck<Mutex<()>, ErrOnOom>);

impl LockedAllocator {
    pub const fn new() -> Self {
        Self(Talc::new(ErrOnOom).lock())
    }

    pub unsafe fn init(&self, heap_start: VirtAddr, heap_size: u64) -> Result<Span, ()> {
        unsafe {
            self.0.lock().claim(Span::new(
                heap_start.as_mut_ptr(),
                (heap_start + heap_size).as_mut_ptr(),
            ))
        }
    }
}

unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        interrupts::without_interrupts(|| unsafe { self.0.alloc(layout) })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        interrupts::without_interrupts(|| unsafe { self.0.dealloc(ptr, layout) })
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        interrupts::without_interrupts(|| unsafe { self.0.alloc_zeroed(layout) })
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        interrupts::without_interrupts(|| unsafe { self.0.realloc(ptr, layout, new_size) })
    }
}
