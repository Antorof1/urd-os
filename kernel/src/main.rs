#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod console;
pub mod display;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod pit;
pub mod task;
pub mod thread;

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

use crate::{
    memory::{
        boot_frame::BootFrameAllocator,
        frame::{PFA, initial_heap_size},
        heap, paging,
        vmm::VMM,
    },
    thread::{Thread, scheduler::SCHEDULER},
};

static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(fb_struct) = boot_info.framebuffer.as_mut() {
        display::init(fb_struct.info(), fb_struct.buffer_mut());
    }

    pit::init();
    gdt::init();
    interrupts::init();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset.take().unwrap());

    let mut boot_frame_allocator = BootFrameAllocator::new(&boot_info.memory_regions);
    let mut page_mapper = unsafe { paging::active_mapper(phys_offset) };

    let frame_count = boot_frame_allocator.frame_count();
    let heap_size = initial_heap_size(frame_count);
    heap::init_boot(&mut boot_frame_allocator, &mut page_mapper, heap_size)
        .expect("boot heap init error");

    PFA.lock().init(frame_count, boot_frame_allocator);
    VMM.lock().init(page_mapper);
    SCHEDULER.lock().init();

    SCHEDULER.lock().spawn(Thread::new(|| {
        println!("Task 1");
        loop {}
    }));

    SCHEDULER.lock().spawn(Thread::new(|| {
        println!("Task 2");
        thread::exit();
    }));

    SCHEDULER.lock().run();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    println!("{}", info);

    loop {
        x86_64::instructions::hlt();
    }
}
