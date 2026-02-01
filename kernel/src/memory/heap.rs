use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB, mapper::MapToError,
    },
};

use crate::memory::{allocator::ALLOCATOR, boot_frame::BootFrameAllocator};

const HEAP_START: VirtAddr = VirtAddr::new(0xFFFF_A000_0000_0000);
pub const MIN_HEAP_GROW: u64 = 64 * 1024;

pub static HEAP_TOP: AtomicU64 = AtomicU64::new(HEAP_START.as_u64());

pub const PAGE_FLAGS: PageTableFlags = PageTableFlags::PRESENT
    .union(PageTableFlags::WRITABLE)
    .union(PageTableFlags::NO_EXECUTE);

pub fn init_boot(
    frame_allocator: &mut BootFrameAllocator,
    page_mapper: &mut OffsetPageTable,
    initial_heap_size: usize,
) -> Result<(), MapToError<Size4KiB>> {
    let first_page = Page::containing_address(HEAP_START);
    let last_page = Page::containing_address(HEAP_START + initial_heap_size as u64 - 1u64);

    let pages_to_map = Page::<Size4KiB>::range_inclusive(first_page, last_page);

    for page in pages_to_map {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        unsafe {
            page_mapper
                .map_to(page, frame, PAGE_FLAGS, frame_allocator)?
                .flush();
        };
    }

    unsafe {
        ALLOCATOR
            .init(HEAP_START, initial_heap_size as u64)
            .map_err(|_| MapToError::FrameAllocationFailed)?;
    }

    HEAP_TOP.fetch_add(initial_heap_size as u64, Ordering::SeqCst);

    Ok(())
}
