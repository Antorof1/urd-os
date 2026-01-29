use alloc::vec::Vec;
use x86_64::{
    align_up,
    structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB},
};

pub struct StackFrameAllocator {
    frames: Vec<PhysFrame>,
}

impl StackFrameAllocator {
    pub fn new(frame_count: usize, frame_iter: impl Iterator<Item = PhysFrame<Size4KiB>>) -> Self {
        let mut frames = Vec::with_capacity(frame_count); // Overallocation just in case
        frames.extend(frame_iter);

        Self { frames }
    }
}

unsafe impl FrameAllocator<Size4KiB> for StackFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.frames.pop()
    }
}

impl FrameDeallocator<Size4KiB> for StackFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.frames.push(frame);
    }
}

pub const fn initial_heap_size(frame_count: usize) -> usize {
    const ADDITIONAL_BUFFER: usize = 1 * 1024 * 1024;

    let vec_size = frame_count * size_of::<PhysFrame>();
    let raw_size = vec_size + ADDITIONAL_BUFFER;

    align_up(raw_size as u64, Size4KiB::SIZE) as usize
}
