use spin::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{gdt::DOUBLE_FAULT_IST_INDEX, println};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(doublefault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    idt
});

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("Got INT3: {:?}", stack_frame);
}

extern "x86-interrupt" fn doublefault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("Got double fault: {:?}", stack_frame);
}
