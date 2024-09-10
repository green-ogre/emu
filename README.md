# emu: RISC-V Emulataor

Native RISC-V emulator in Rust.

## Resources
  - Spec: [RISC-V instruction set spec]([https://github.com/d0iasm/rvemu/blob/main/README.md?plain=1](https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf))
  - Tests: [rvemu](https://github.com/d0iasm/rvemu/blob/main/README.md?plain=1)

## Usage

The emulator can be run nativly on any platform with rustup. The emulator can compile and run RISC-V assembly nativley on Linux (See **Linux** below for dependencies).

**Run binary**

You can can run the emulator with or without a binary, specified by `--file` or `-f`.
Emu will default to a Fibinacci demo.
```
$ ./target/release/emu [-f <your-binary>.bin]
```

**Linux**

**Compile and run binary**

In order to assemble a .S file, Emu uses the RISC-V toolchain, specifically riscv64-unknown-elf-gcc.
To download the prebuilt binaries, refer to the [riscv-gcc-prebuilt](https://github.com/stnolting/riscv-gcc-prebuilt) repo.

You can use any RISC-V compliant assembly file, specified by `--file` or `f`. Optionally, you can specify where in memory the program will be flashed with `--location` or `l`.
The resulting ELF will be assembled and stripped of headers.
```
$ ./target/release/emu [-l <hex>] -cr <your-assembly>.S
```

## Build

You can build Emu by using the rust toolchain.

On Unix system:
```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Refer to the rustup book for Windows: [rust-lang](https://rust-lang.github.io/rustup/installation/other.html)

Clone the repository, then build the crate:
```
$ git clone https://github.com/green-ogre/emu
$ cd ./emu
$ cargo build --release
```

## Features List

The emulator supports the following features:
- [] RV64G ISA
  - [x] RV64I (v2.1): supports 43/52 instructions
  - [] RV64M (v2.0): supports 0/13 instructions
  - [] RV64A (v2.1): supports 0/22 instructions
  - [] RV64F (v2.2): supports 0/30 instructions
  - [] RV64D (v2.2): supports 0/32 instructions

## Dependencies

- [Rust]([https://doc.rust-lang.org/1.2.0/book/nightly-rust.html](https://www.rust-lang.org/))

### Helpful Tools

- [RISC-V Online Simulator](https://www.kvakil.me/venus/)
