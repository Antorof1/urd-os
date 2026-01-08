use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB, mapper::MapToError,
    },
};

use crate::memory::{allocator::ALLOCATOR, boot_frame::BootFrameAllocator};

const HEAP_START: VirtAddr = VirtAddr::new(0x7777_7777_0000);
const HEAP_SIZE: u64 = 5 * 1024 * 1024;

const PAGE_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE);

pub fn init_boot(
    frame_allocator: &mut BootFrameAllocator,
    page_mapper: &mut OffsetPageTable,
) -> Result<(), MapToError<Size4KiB>> {
    let first_page = Page::containing_address(HEAP_START);
    let last_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1u64);

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
            .init(HEAP_START, HEAP_SIZE)
            .map_err(|_| MapToError::FrameAllocationFailed)?;
    }

    Ok(())
}
