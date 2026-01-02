use pic8259::ChainedPics;
use spin::{Lazy, Mutex};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{gdt::DOUBLE_FAULT_IST_INDEX, println};

const PIC_MASTER_OFFSET: u8 = 32;
const PIC_SLAVE_OFFSET: u8 = PIC_MASTER_OFFSET + 8;

#[repr(u8)]
enum IRQIndex {
    Timer,
}

impl IRQIndex {
    fn with_offset(self) -> u8 {
        self as u8 + PIC_MASTER_OFFSET
    }

    fn as_bitmask(self) -> u16 {
        1 << (self as u8)
    }
}

static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_MASTER_OFFSET, PIC_SLAVE_OFFSET) });

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(doublefault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }
    idt[IRQIndex::Timer.with_offset()].set_handler_fn(timer_handler);

    idt
});

pub fn init() {
    IDT.load();

    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();

        let enabled_irqs: u16 = IRQIndex::Timer.as_bitmask();

        let all_masks = !enabled_irqs;

        let master_mask = all_masks as u8;
        let slave_mask = (all_masks >> 8) as u8;

        pics.write_masks(master_mask, slave_mask);
    }

    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("Got INT3: {:#?}", stack_frame);
}

extern "x86-interrupt" fn doublefault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("Got double fault: {:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(IRQIndex::Timer.with_offset());
    }
}
