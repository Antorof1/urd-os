# Urd OS

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-x86__64-lightgrey.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)]()

Urd OS is a hobby operating system written in Rust for the `x86_64` architecture. It aims to explore modern OS design patterns using Rust's safety and concurrency features.

## Features

- **Architecture**: 64-bit `x86_64` with support for modern CPU features.
- **Booting**: Boots on UEFI systems using the [bootloader](https://github.com/rust-osdev/bootloader) crate.
- **Memory Management**:
    - **Physical Memory Mapping**: Full physical memory access via offset mapping.
    - **Frame Allocation**: Efficient stack-based physical frame allocator.
    - **Virtual Memory Manager (VMM)**: Support for demand paging and range allocation.
    - **Heap Allocation**: High-performance heap allocation using the [talc](https://github.com/SFBdragon/talc) allocator.
- **Concurrency & Multitasking**:
    - **Kernel Threads**: Support for spawning and managing kernel threads.
    - **Context Switching**: Manual x86_64 context switching and stack management.
    - **Scheduler**: A cooperative/preemptive scheduler for managing multiple tasks.
    - **Async/Await**: Native support for asynchronous tasks with a custom executor.
- **Hardware Support**:
    - **GDT & IDT**: Global Descriptor Table and Interrupt Descriptor Table management.
    - **PIT**: Programmable Interval Timer for system time and scheduling.
    - **Serial Console**: Logging and debugging output via UART 16550.
    - **Graphics**: Framebuffer display support with [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics) integration.

## Getting Started

### Prerequisites

To build and run Urd OS, you need:

1.  **Rust Nightly**: The project uses several unstable features.
    ```bash
    rustup override set nightly
    rustup component add rust-src llvm-tools-preview
    ```
2.  **QEMU**: For emulation.
    ```bash
    # On Ubuntu/Debian
    sudo apt install qemu-system-x86
    ```

### Running

The project includes a custom runner that builds the kernel and launches QEMU.

```bash
# Clone the repository
git clone https://github.com/Antorof1/urd-os
cd urd-os

# Run in QEMU
cargo run
```

#### Run Options

- `--display`: Launches QEMU with a graphical window (GTK).
- `--gdb`: Starts QEMU with a GDB server on port `1234` and freezes execution until a debugger connects.

Example:
```bash
cargo run -- --display --gdb
```

## Project Structure

- `kernel/`: The core operating system.
    - `main.rs`: Kernel entry point and subsystem initialization.
    - `memory/`: Physical frame allocation, paging, and virtual memory management (VMM).
    - `process/` & `sync/`: Thread management, multitasking, and synchronization primitives.
    - `task/`: Support for `async/await`, including the executor and Waker implementation.
    - `console/` & `display/`: Drivers for VGA/Framebuffer graphics and serial port (UART) output.
    - `interrupts.rs`, `gdt.rs` & `pit.rs`: Low-level x86_64 CPU configuration, hardware interrupts, and timing.
- `src/`: Host-side tooling.
    - `main.rs`: A custom runner that manages QEMU orchestration and UEFI firmware (OVMF).
- `build.rs`: Glue logic that uses the `bootloader` crate to package the kernel into bootable disk images.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
