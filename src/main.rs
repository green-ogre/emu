mod emulator;

fn main() {
    // riscv32-unknown-elf-gcc -nostdlib -Ttext=0x80000000 -o emu.txt ./emu.S
    // let output = Command::new("riscv32-unknown-elf-gcc")
    //     .arg("-nostdlib")
    //     .arg("-Ttext=0x80000000")
    //     .arg("-oemu.txt")
    //     .arg("./emu.S")
    //     .output()
    //     .unwrap();
    // if !output.stdout.is_empty() {
    //     println!("{}", String::from_utf8_lossy(&output.stdout));
    // }
    // if !output.stderr.is_empty() {
    //     println!("{}", String::from_utf8_lossy(&output.stderr));
    // }
    // // riscv32-unknown-elf-objcopy -O binary emu.bin
    // let output = Command::new("riscv32-unknown-elf-objcopy")
    //     .arg("-O")
    //     .arg("binary")
    //     .arg("emu.txt")
    //     .output()
    //     .unwrap();
    // if !output.stdout.is_empty() {
    //     println!("{}", String::from_utf8_lossy(&output.stdout));
    // }
    // if !output.stderr.is_empty() {
    //     println!("{}", String::from_utf8_lossy(&output.stderr));
    // }

    let raw = std::fs::read("./emu.txt").unwrap();

    // std::fs::remove_file("./emu.txt").unwrap();

    emulator::emulate(&raw);
}
