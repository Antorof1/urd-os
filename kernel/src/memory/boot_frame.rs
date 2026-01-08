use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    PhysAddr, align_up,
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
};

pub struct BootFrameAllocator<'a> {
    regions: &'a MemoryRegions,

    last_region: usize,
    memory_offset: u64,
}

impl<'a> BootFrameAllocator<'a> {
    pub fn new(regions: &'a MemoryRegions) -> Self {
        Self {
            regions,
            last_region: 0,
            memory_offset: 0,
        }
    }

    pub fn frame_count(&self) -> usize {
        self.regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .map(|r| (r.end - r.start) / 4096)
            .sum::<u64>() as usize
    }
}

unsafe impl<'a> FrameAllocator<Size4KiB> for BootFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        loop {
            if self.last_region == self.regions.len() {
                return None;
            }

            let region = self.regions[self.last_region];

            if region.kind == MemoryRegionKind::Usable {
                let current_address = align_up(region.start, 4096) + self.memory_offset;

                if current_address + 4096 <= region.end {
                    self.memory_offset += 4096;

                    let address = PhysAddr::new(current_address);

                    unsafe {
                        return Some(PhysFrame::from_start_address_unchecked(address));
                    }
                }

                self.memory_offset = 0;
            }

            self.last_region += 1;
        }
    }
}

impl<'a> Iterator for BootFrameAllocator<'a> {
    type Item = PhysFrame;

    fn next(&mut self) -> Option<Self::Item> {
        self.allocate_frame()
    }
}
