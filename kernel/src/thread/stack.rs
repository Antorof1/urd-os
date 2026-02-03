use x86_64::{
    VirtAddr,
    structures::paging::{PageSize, PageTableFlags, Size4KiB, mapper::MapToError},
};

use crate::memory;

const STACK_PAGE_FLAGS: PageTableFlags = PageTableFlags::PRESENT
    .union(PageTableFlags::WRITABLE)
    .union(PageTableFlags::NO_EXECUTE);

pub struct Stack {
    bottom_address: VirtAddr,
    size: u64,
}

impl Stack {
    pub fn new(bottom_address: VirtAddr, size: u64) -> Result<Self, MapToError<Size4KiB>> {
        let stack_data_start = bottom_address + Size4KiB::SIZE;

        // Ignore if already unmapped
        let _ = memory::dealloc_page(bottom_address);
        memory::alloc_range(stack_data_start, size, STACK_PAGE_FLAGS)?;

        Ok(Self {
            bottom_address,
            size,
        })
    }

    pub fn top(&self) -> VirtAddr {
        self.bottom_address + Size4KiB::SIZE + self.size
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        let stack_data_start = self.bottom_address + Size4KiB::SIZE;

        let _ = memory::dealloc_range(stack_data_start, self.size);
    }
}
