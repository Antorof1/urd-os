use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
};

pub unsafe fn active_mapper(phys_offset: VirtAddr) -> OffsetPageTable<'static> {
    let phys_table_address = Cr3::read().0;
    let virt_table_address =
        VirtAddr::new(phys_table_address.start_address().as_u64() + phys_offset.as_u64());
    let table: &mut PageTable = unsafe { &mut *virt_table_address.as_mut_ptr() };

    unsafe { OffsetPageTable::new(table, phys_offset) }
}
