use spin::Mutex;
use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
        mapper::MapToError,
    },
};

use crate::memory::frame::PFA;

pub static VMM: Mutex<VirtualMemoryManager> = Mutex::new(VirtualMemoryManager::new());

pub struct VirtualMemoryManager {
    mapper: Option<OffsetPageTable<'static>>,
}

impl VirtualMemoryManager {
    pub const fn new() -> Self {
        Self { mapper: None }
    }

    pub fn init(&mut self, mapper: OffsetPageTable<'static>) {
        self.mapper = Some(mapper);
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
            let result = self.mapper_mut().map_to(page, frame, flags, &mut *pfa);

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

    fn mapper_mut(&mut self) -> &mut OffsetPageTable<'static> {
        self.mapper.as_mut().expect("VMM is not initialized")
    }
}
