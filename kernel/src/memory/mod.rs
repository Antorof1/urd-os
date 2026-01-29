use x86_64::{
    VirtAddr,
    structures::paging::{
        PageTableFlags, Size4KiB,
        mapper::{MapToError, UnmapError},
    },
};

use crate::memory::vmm::vmm;

pub mod allocator;
pub mod boot_frame;
pub mod frame;
pub mod heap;
pub mod paging;
pub mod vmm;

pub fn alloc_page(address: VirtAddr, flags: PageTableFlags) -> Result<(), MapToError<Size4KiB>> {
    vmm().lock(|vmm| vmm.alloc_page(address, flags))
}

pub fn alloc_range(
    address: VirtAddr,
    size: u64,
    flags: PageTableFlags,
) -> Result<(), MapToError<Size4KiB>> {
    vmm().lock(|vmm| vmm.alloc_range(address, size, flags))
}

pub fn dealloc_page(address: VirtAddr) -> Result<(), UnmapError> {
    vmm().lock(|vmm| vmm.dealloc_page(address))
}

pub fn dealloc_range(address: VirtAddr, size: u64) -> Result<(), UnmapError> {
    vmm().lock(|vmm| vmm.dealloc_range(address, size))
}
