use x86_64::{
    registers::control::Cr3,
    structures::paging::{PageTable, PhysFrame, Size4KiB},
};

use crate::memory::vmm::{VirtualMemoryManager, vmm};

#[derive(Debug)]
pub struct ProcessAddressSpace {
    p4_table: PhysFrame<Size4KiB>,
}

impl ProcessAddressSpace {
    pub fn new(vmm: &mut VirtualMemoryManager) -> Option<Self> {
        let table_phys_addr = vmm.alloc_frame()?;

        unsafe {
            let table_addr = vmm.phys_to_virt(table_phys_addr.start_address());
            let table = &mut *table_addr.as_mut_ptr::<PageTable>();

            table.zero();

            let current_p4_frame = Cr3::read().0;
            let current_p4_addr = vmm.phys_to_virt(current_p4_frame.start_address());
            let current_table = &mut *current_p4_addr.as_mut_ptr::<PageTable>();

            // Copy kernel higher half
            for i in 256..512 {
                table[i] = current_table[i].clone();
            }
        }

        Some(Self {
            p4_table: table_phys_addr,
        })
    }

    pub fn from_current() -> Self {
        let current_table_frame = Cr3::read().0;

        Self {
            p4_table: current_table_frame,
        }
    }

    pub fn cr3_value(&self) -> u64 {
        self.p4_table.start_address().as_u64()
    }
}

impl Drop for ProcessAddressSpace {
    fn drop(&mut self) {
        vmm().lock(|vmm| vmm.dealloc_frame(self.p4_table));
    }
}
