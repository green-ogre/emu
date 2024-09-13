use std::process::Command;

mod emulator;

fn main() {
    let output = Command::new("riscv64-unknown-elf-gcc")
        .arg("-S")
        .arg("-nostdlib")
        .arg("-march=rv64g")
        .arg("-Wall")
        .arg("-O3")
        .arg("emu.c")
        .output()
        .unwrap();
    if !output.stdout.is_empty() {
        println!("compiler: {}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        println!("compiler: {}", String::from_utf8_lossy(&output.stderr));
    }

    let output = Command::new("riscv64-unknown-elf-gcc")
        .arg("-nostdlib")
        .arg("-emain")
        .arg("-Wl,-Tmain_linker.ld")
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

    let objdump_output = Command::new("riscv64-unknown-elf-objdump")
        .arg("-d")
        .arg("emul")
        .output()
        .unwrap();

    let raw = std::fs::read("./pgrm").unwrap();
    let emulator = emulator::run_emulator(&raw);

    println!(
        "\nDisassembly:\n{}\n",
        String::from_utf8_lossy(&objdump_output.stdout)
    );

    emulator::print_emulator(&emulator);

    std::fs::remove_file("emul").unwrap();
    std::fs::remove_file("pgrm").unwrap();
    std::fs::remove_file("emu.s").unwrap();
}
