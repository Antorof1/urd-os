use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, Page, PageTableFlags, Size4KiB, mapper::MapToError},
};

use crate::memory::{boot_frame::BootFrameAllocator, page::PageMapper};

const HEAP_START: VirtAddr = VirtAddr::new(0x7777_7777_0000);
const HEAP_SIZE: u64 = 100 * 1024;

const PAGE_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE);

pub fn init_boot(
    frame_allocator: &mut BootFrameAllocator,
    page_mapper: &mut PageMapper,
) -> Result<(), MapToError<Size4KiB>> {
    let first_page = Page::containing_address(HEAP_START);
    let last_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1u64);

    let pages_to_map = Page::<Size4KiB>::range_inclusive(first_page, last_page);

    for page in pages_to_map {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        unsafe { page_mapper.map_page(page, frame, PAGE_FLAGS, frame_allocator)? };
    }

    Ok(())
}
