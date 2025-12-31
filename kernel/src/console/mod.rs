use bootloader_api::info::FrameBufferInfo;
use spin::{Lazy, Mutex};
use uart_16550::SerialPort;

use crate::console::framebuffer::FramebufferDriver;

pub mod framebuffer;

pub static CONSOLE: Lazy<Mutex<Console>> = Lazy::new(|| Mutex::new(Console::default()));

pub struct Console {
    serial: SerialPort,
    framebuffer: Option<FramebufferDriver>,
}

impl Console {
    pub fn init_framebuffer(&mut self, info: FrameBufferInfo, buffer: &mut [u8]) {
        let bytes = unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr(), buffer.len()) };

        let mut fb_driver = FramebufferDriver::new(info, bytes);

        fb_driver.clear_screen();

        self.framebuffer = Some(fb_driver);
    }
}

impl Default for Console {
    fn default() -> Self {
        const SERIAL_IO_PORT: u16 = 0x3F8;

        let mut serial = unsafe { SerialPort::new(SERIAL_IO_PORT) };
        serial.init();

        Self {
            serial,
            framebuffer: None,
        }
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    let mut console = CONSOLE.lock();

    let _ = console.serial.write_fmt(args);

    if let Some(fb) = &mut console.framebuffer {
        let _ = fb.write_fmt(args);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
