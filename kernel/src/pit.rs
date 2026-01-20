use x86_64::instructions::port::Port;

pub fn init() {
    // 1000 Hz
    let divisor = 1193u16;

    let divisor_low = (divisor & 0xFF) as u8;
    let divisor_high = ((divisor >> 8) & 0xFF) as u8;

    let mut command_port = Port::new(0x43);
    let mut data_port = Port::new(0x40);

    unsafe {
        command_port.write(0x36u8);

        data_port.write(divisor_low);
        data_port.write(divisor_high);
    }
}
