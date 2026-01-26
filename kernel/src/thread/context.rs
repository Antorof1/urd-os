use core::arch::naked_asm;

use x86_64::VirtAddr;

#[repr(C)]
#[derive(Debug, Default)]
pub struct ContextFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[macro_export]
macro_rules! save_context {
    () => {
        concat!(
            r#"
			push rax
			push rbx
			push rcx
			push rdx
			push rsi
			push rdi
			push rbp
			push r8
			push r9
			push r10
			push r11
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
			pop r11
			pop r10
			pop r9
			pop r8
			pop rbp
			pop rdi
			pop rsi
			pop rdx
			pop rcx
			pop rbx
			pop rax
			"#
        )
    };
}

#[unsafe(naked)]
pub unsafe extern "C" fn switch_to_context(target_stack: VirtAddr) {
    // rdi -> target stack address

    naked_asm!("mov rsp, rdi", restore_context!(), "iretq");
}
