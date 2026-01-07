#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod console;
pub mod display;
pub mod gdt;
pub mod interrupts;
pub mod memory;

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

use crate::memory::{boot_frame::BootFrameAllocator, heap, page::PageMapper};

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

    gdt::init();
    interrupts::init();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset.take().unwrap());

    let mut frame_allocator = BootFrameAllocator::new(&boot_info.memory_regions);
    let mut page_mapper = unsafe { PageMapper::from_cr3(phys_offset) };

    heap::init_boot(&mut frame_allocator, &mut page_mapper).expect("boot heap init error");

    println!("Begin Operation Urd!\n\n");

    println!("{boot_info:#?}");

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {
        x86_64::instructions::hlt();
    }
}
