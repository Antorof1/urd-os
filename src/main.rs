use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::env;
use std::process::{Command, exit};

fn main() {
    let uefi_path = env!("UEFI_PATH");
    let args: Vec<String> = env::args().collect();

    let use_gdb = args.iter().any(|arg| arg == "--gdb");
    let use_display = args.iter().any(|arg| arg == "--display");

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("Usage: cargo run [options]");
        println!("Options:");
        println!("  --gdb      - Start QEMU with GDB server on port 1234 and freeze execution");
        println!("  --display  - Show graphical window (GTK)");
        exit(0);
    }

    let mut cmd = Command::new("qemu-system-x86_64");

    if use_gdb {
        cmd.arg("-s").arg("-S");
    }

    if use_display {
        cmd.arg("-display").arg("gtk");
    } else {
        cmd.arg("-display").arg("none");
    }

    cmd.arg("-m").arg("4G");
    cmd.arg("-serial").arg("mon:stdio");
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    let prebuilt =
        Prebuilt::fetch(Source::LATEST, "target/ovmf").expect("failed to update prebuilt");

    let code = prebuilt.get_file(Arch::X64, FileType::Code);
    let vars = prebuilt.get_file(Arch::X64, FileType::Vars);

    cmd.arg("-drive")
        .arg(format!("format=raw,file={uefi_path}"));
    cmd.arg("-drive").arg(format!(
        "if=pflash,format=raw,unit=0,file={},readonly=on",
        code.display()
    ));
    cmd.arg("-drive").arg(format!(
        "if=pflash,format=raw,unit=1,file={},snapshot=on",
        vars.display()
    ));

    let mut child = cmd.spawn().expect("failed to start qemu-system-x86_64");
    let status = child.wait().expect("failed to wait on qemu");

    match status.code().unwrap_or(1) {
        0x10 => 0, // success
        0x11 => 1, // failure
        _ => 2,    // unknown fault
    };
}
