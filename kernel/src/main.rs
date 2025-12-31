#![no_std]
#![no_main]

pub mod console;

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

    println!("Begin Operation Urd!\n\n");

    println!("{boot_info:?}");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
