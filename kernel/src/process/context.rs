use core::arch::naked_asm;

use x86_64::VirtAddr;

#[repr(C)]
#[derive(Debug, Default)]
pub struct Context {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,

    pub rip: u64,
}

#[macro_export]
macro_rules! save_context {
    () => {
        concat!(
            r#"
			push rbx
			push rbp
			push r12
			push r13
			push r14
			push r15
			"#,
        )
    };
}

#[macro_export]
macro_rules! restore_context {
    () => {
        concat!(
            r#"
			pop r15
			pop r14
			pop r13
			pop r12
			pop rbp
			pop rbx
			"#
        )
    };
}

#[unsafe(naked)]
pub unsafe extern "C" fn switch_to_context(target_stack: VirtAddr) {
    naked_asm!("mov rsp, rdi", restore_context!(), "ret");
}

#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(
    current_stack_ptr: *mut VirtAddr,
    next_stack: VirtAddr,
    next_cr3: u64,
) {
    naked_asm!(
        save_context!(),
        "mov [rdi], rsp",
        "mov rsp, rsi",
        "mov rax, cr3",
        "cmp rax, rdx", // Compare current cr3 with next_cr3
        "je 1f",
        "mov cr3, rdx",
        "1:",
        restore_context!(),
        "ret"
    );
}
