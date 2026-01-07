use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB, mapper::MapToError,
    },
};

pub struct PageMapper<'a> {
    mapper: OffsetPageTable<'a>,
}

impl<'a> PageMapper<'a> {
    pub unsafe fn new(level_4_table: &'a mut PageTable, phys_offset: VirtAddr) -> Self {
        unsafe {
            Self {
                mapper: OffsetPageTable::new(level_4_table, phys_offset),
            }
        }
    }

    pub unsafe fn from_cr3(phys_offset: VirtAddr) -> Self {
        let phys_table_address = Cr3::read().0;
        let virt_table_address =
            VirtAddr::new(phys_table_address.start_address().as_u64() + phys_offset.as_u64());
        let table: &mut PageTable = unsafe { &mut *virt_table_address.as_mut_ptr() };

        unsafe { Self::new(table, phys_offset) }
    }

    pub unsafe fn map_page(
        &mut self,
        page: Page,
        frame: PhysFrame,
        flags: PageTableFlags,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    ) -> Result<(), MapToError<Size4KiB>> {
        unsafe {
            self.mapper
                .map_to(page, frame, flags, frame_allocator)?
                .flush();
        }

        Ok(())
    }
}
