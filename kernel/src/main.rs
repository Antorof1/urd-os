#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use core::fmt::Write;
use core::panic::PanicInfo;
use uart_16550::SerialPort;

fn serial() -> SerialPort {
    const SERIAL_IO_PORT: u16 = 0x3F8;

    let mut port = unsafe { SerialPort::new(SERIAL_IO_PORT) };
    port.init();

    port
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut serial_port = serial();

    writeln!(serial_port, "{boot_info:?}").unwrap();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
