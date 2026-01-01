#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod console;
pub mod gdt;
pub mod interrupts;

use bootloader_api::{BootInfo, entry_point};
use core::panic::PanicInfo;

use crate::console::CONSOLE;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(fb_struct) = boot_info.framebuffer.as_mut() {
        CONSOLE
            .lock()
            .init_framebuffer(fb_struct.info(), fb_struct.buffer_mut());
    }

    gdt::init();
    interrupts::init();

    println!("Begin Operation Urd!\n\n");

    println!("{boot_info:?}");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}
