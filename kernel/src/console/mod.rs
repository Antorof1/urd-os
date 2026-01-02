use spin::{Lazy, Mutex};
use uart_16550::SerialPort;

use crate::console::display::DisplayConsole;

mod display;

pub static CONSOLE: Lazy<Mutex<Console>> = Lazy::new(|| Mutex::new(Console::new()));

pub struct Console {
    serial: SerialPort,
    display: Option<DisplayConsole>,
}

impl Console {
    fn new() -> Self {
        const SERIAL_IO_PORT: u16 = 0x3F8;

        let mut serial = unsafe { SerialPort::new(SERIAL_IO_PORT) };
        serial.init();

        let display = crate::display::DISPLAY
            .lock()
            .is_some()
            .then(DisplayConsole::default);

        Self { serial, display }
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut console = CONSOLE.lock();

        let _ = console.serial.write_fmt(args);

        if let Some(display) = &mut console.display {
            let _ = display.write_fmt(args);
        }
    });
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
