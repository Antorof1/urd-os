use core::{
    alloc::{GlobalAlloc, Layout},
    cmp,
    sync::atomic::Ordering,
};

use spin::Mutex;
use talc::{OomHandler, Span, Talc, Talck};
use x86_64::{
    VirtAddr, align_up,
    instructions::interrupts,
    structures::paging::{PageSize, Size4KiB},
};

use crate::memory::{heap, vmm::VMM};

#[global_allocator]
pub static ALLOCATOR: LockedAllocator = LockedAllocator::new();

pub struct LockedAllocator(Talck<Mutex<()>, HeapOomHandler>);

impl LockedAllocator {
    pub const fn new() -> Self {
        Self(Talc::new(HeapOomHandler).lock())
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

struct HeapOomHandler;

impl OomHandler for HeapOomHandler {
    fn handle_oom(talc: &mut Talc<Self>, layout: Layout) -> Result<(), ()> {
        let requested_size = layout.size() as u64;
        let growth_size = cmp::max(requested_size, heap::MIN_HEAP_GROW);

        let size_to_map = align_up(growth_size, Size4KiB::SIZE);
        let required_pages = size_to_map / Size4KiB::SIZE;

        let start_addr = VirtAddr::new(heap::HEAP_TOP.fetch_add(size_to_map, Ordering::SeqCst));

        let mut vmm = VMM.lock();

        for i in 0..required_pages {
            vmm.alloc_page(start_addr + (i * Size4KiB::SIZE), heap::PAGE_FLAGS)
                .map_err(|_| ())?;
        }

        unsafe {
            talc.claim(Span::from_base_size(
                start_addr.as_mut_ptr(),
                size_to_map as usize,
            ))
            .map_err(|_| ())?;
        }

        Ok(())
    }
}
