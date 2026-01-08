use alloc::vec::Vec;
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};

pub static PFA: Mutex<StackFrameAllocator> = Mutex::new(StackFrameAllocator::new());

pub struct StackFrameAllocator {
    frames: Vec<PhysFrame>,
}

impl StackFrameAllocator {
    pub const fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn init(
        &mut self,
        frame_count: usize,
        frame_iter: impl Iterator<Item = PhysFrame<Size4KiB>>,
    ) {
        self.frames.reserve(frame_count);
        self.frames.extend(frame_iter);
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

pub fn initial_heap_size(frame_count: usize) -> usize {
    const ADDITIONAL_BUFFER: usize = 1 * 1024 * 1024;

    let vec_size = frame_count * size_of::<PhysFrame>();

    vec_size + ADDITIONAL_BUFFER
}
