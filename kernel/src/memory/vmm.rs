use spin::Once;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame,
        Size4KiB,
        mapper::{MapToError, UnmapError},
    },
};

use crate::{memory::frame::StackFrameAllocator, sync::IrqLock};

static VMM: Once<IrqLock<VirtualMemoryManager>> = Once::new();

pub(crate) fn vmm() -> &'static IrqLock<VirtualMemoryManager> {
    VMM.get().expect("VMM called before init()")
}

pub fn init(mapper: OffsetPageTable<'static>, frame_allocator: StackFrameAllocator) {
    VMM.call_once(|| IrqLock::new(VirtualMemoryManager::new(mapper, frame_allocator)));
}

pub struct VirtualMemoryManager {
    mapper: OffsetPageTable<'static>,
    frame_allocator: StackFrameAllocator,
}

impl VirtualMemoryManager {
    pub fn new(mapper: OffsetPageTable<'static>, frame_allocator: StackFrameAllocator) -> Self {
        Self {
            mapper,
            frame_allocator,
        }
    }

    pub fn alloc_page(
        &mut self,
        address: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<Size4KiB>> {
        let frame = self
            .frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let page = Page::containing_address(address);

        unsafe {
            let result = self
                .mapper
                .map_to(page, frame, flags, &mut self.frame_allocator);

            match result {
                Ok(mapper_flush) => {
                    mapper_flush.flush();
                    Ok(())
                }
                Err(e) => {
                    self.frame_allocator.deallocate_frame(frame);
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

        for i in 0..pages.count() {
            let page = first_page + i as u64;

            let frame = self
                .frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            unsafe {
                match self
                    .mapper
                    .map_to(page, frame, flags, &mut self.frame_allocator)
                {
                    Ok(tlb) => tlb.flush(),
                    Err(e) => {
                        // Deallocate current frame
                        self.frame_allocator.deallocate_frame(frame);

                        // Deallocate previous frames
                        for j in 0..i {
                            let page_to_unmap = first_page + j as u64;

                            match self.mapper.unmap(page_to_unmap) {
                                Ok((mapped_frame, flush)) => {
                                    flush.flush();
                                    self.frame_allocator.deallocate_frame(mapped_frame);
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

    pub fn alloc_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.frame_allocator.allocate_frame()
    }

    pub fn dealloc_page(&mut self, address: VirtAddr) -> Result<(), UnmapError> {
        let page = Page::<Size4KiB>::containing_address(address);

        let (frame, flush) = self.mapper.unmap(page)?;
        flush.flush();

        unsafe {
            self.frame_allocator.deallocate_frame(frame);
        }

        Ok(())
    }

    pub fn dealloc_range(&mut self, address: VirtAddr, size: u64) -> Result<(), UnmapError> {
        let first_page = Page::containing_address(address);
        let last_page = Page::containing_address(address + size - 1u64);
        let pages = Page::<Size4KiB>::range_inclusive(first_page, last_page);

        for page in pages {
            // Stop on error if caller fucked up
            let (frame, flush) = self.mapper.unmap(page)?;
            flush.flush();

            unsafe {
                self.frame_allocator.deallocate_frame(frame);
            }
        }

        Ok(())
    }

    pub fn dealloc_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        unsafe { self.frame_allocator.deallocate_frame(frame) };
    }

    pub fn phys_to_virt(&self, phys: PhysAddr) -> VirtAddr {
        VirtAddr::new(phys.as_u64() + self.mapper.phys_offset().as_u64())
    }
}
