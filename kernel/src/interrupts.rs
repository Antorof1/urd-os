use core::arch::naked_asm;

use pic8259::ChainedPics;
use spin::{Lazy, Mutex};
use x86_64::{
    VirtAddr,
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

use crate::{
    gdt::DOUBLE_FAULT_IST_INDEX,
    println, restore_context, save_context,
    task::timer::on_timer_tick,
    thread::context::{ContextFrame, try_switch_threads},
};

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
    idt.general_protection_fault.set_handler_fn(gpf_handler);
    idt.page_fault.set_handler_fn(pagefault_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(doublefault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);

        idt[IRQIndex::Timer.with_offset()]
            .set_handler_addr(VirtAddr::new(timer_handler as *const () as u64));
    }

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
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn gpf_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n\
        Error Code: {:#x}\n\
        {:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn pagefault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "EXCEPTION: PAGE FAULT\n\
        Accessed Address: {:?}\n\
        Error Code: {:?}\n\
        {:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn doublefault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[unsafe(naked)]
extern "C" fn timer_handler() {
    // SS, RSP, RFLAGS, CS and RIP are pushed to current thread's stack by hardware in ContextFrame order
    naked_asm!(
        // Push preserved and scratch registers to current thread's stack in ContextFrame order
        save_context!(),
        // Pass current thread's stack pointer to the inner function
        "mov rdi, rsp",
        // Returns same RSP, or next thread's RSP if needed schedule
        "call timer_handler_inner",
        // Switch stack to returned pointer
        "mov rsp, rax",
        // Pop preserved and scratch registers from next thread's stack in ContextFrame order
        restore_context!(),
        // SS, RSP, RFLAGS, CS and RIP are popped from next thread's stack in ContextFrame order
        "iretq"
    );
}

#[unsafe(no_mangle)]
extern "C" fn timer_handler_inner(current_frame: *mut ContextFrame) -> *mut ContextFrame {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(IRQIndex::Timer.with_offset());
    }

    on_timer_tick();

    try_switch_threads(current_frame)
}
