use std::process::Command;

mod emulator;

fn main() {
    let output = Command::new("riscv64-unknown-elf-gcc")
        .arg("-S")
        .arg("-nostdlib")
        .arg("-march=rv64g")
        .arg("emu.c")
        .output()
        .unwrap();
    if !output.stdout.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    let output = Command::new("riscv64-unknown-elf-gcc")
        .arg("-nostdlib")
        .arg("-emain")
        .arg("-Wl,-Ttext=0x40000000")
        .arg("-march=rv64g")
        .arg("-o")
        .arg("emul")
        .arg("emu.s")
        .output()
        .unwrap();
    if !output.stdout.is_empty() {
        println!("assembler: {}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        println!("assembler: {}", String::from_utf8_lossy(&output.stderr));
    }

    let output = Command::new("riscv64-unknown-elf-objcopy")
        .arg("-O")
        .arg("binary")
        .arg("emul")
        .arg("pgrm")
        .output()
        .unwrap();
    if !output.stdout.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    let raw = std::fs::read("./pgrm").unwrap();

    // std::fs::remove_file("./emu.txt").unwrap();

    emulator::emulate(&raw);
}
