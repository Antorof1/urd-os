use spin::Once;
use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
        mapper::MapToError,
    },
};

use crate::{memory::frame::PFA, sync::IrqLock};

static VMM: Once<IrqLock<VirtualMemoryManager>> = Once::new();

pub(crate) fn vmm() -> &'static IrqLock<VirtualMemoryManager> {
    VMM.get().expect("VMM called before init()")
}

pub fn init(mapper: OffsetPageTable<'static>) {
    VMM.call_once(|| IrqLock::new(VirtualMemoryManager::new(mapper)));
}

pub struct VirtualMemoryManager {
    mapper: OffsetPageTable<'static>,
}

impl VirtualMemoryManager {
    pub fn new(mapper: OffsetPageTable<'static>) -> Self {
        Self { mapper }
    }

    pub fn alloc_page(
        &mut self,
        address: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<Size4KiB>> {
        let mut pfa = PFA.lock();

        let frame = pfa
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let page = Page::containing_address(address);

        unsafe {
            let result = self.mapper.map_to(page, frame, flags, &mut *pfa);

            match result {
                Ok(mapper_flush) => {
                    mapper_flush.flush();
                    Ok(())
                }
                Err(e) => {
                    pfa.deallocate_frame(frame);
                    Err(e)
                }
            }
        }
    }

    pub fn alloc_range(
        &mut self,
        address: VirtAddr,
        size: u64,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<Size4KiB>> {
        let first_page = Page::containing_address(address);
        let last_page = Page::containing_address(address + size - 1u64);
        let pages = Page::<Size4KiB>::range_inclusive(first_page, last_page);

        let mut pfa = PFA.lock();

        for i in 0..pages.count() {
            let page = first_page + i as u64;

            let frame = pfa
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            unsafe {
                match self.mapper.map_to(page, frame, flags, &mut *pfa) {
                    Ok(tlb) => tlb.flush(),
                    Err(e) => {
                        // Deallocate current frame
                        pfa.deallocate_frame(frame);

                        // Deallocate previous frames
                        for j in 0..i {
                            let page_to_unmap = first_page + j as u64;

                            match self.mapper.unmap(page_to_unmap) {
                                Ok((mapped_frame, flush)) => {
                                    flush.flush();
                                    pfa.deallocate_frame(mapped_frame);
                                }
                                Err(_) => {
                                    panic!("VMM: Failed to unmap page during cleanup rollback")
                                }
                            }
                        }

                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }
}
