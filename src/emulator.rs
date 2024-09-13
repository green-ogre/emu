use crate::instruction_set::*;
use crate::primitives::*;

pub const CONSOLE_OFFSET: u64 = 0x4;
pub const HEAP_OFFSET: u64 = 0x8;
pub const NULL: u64 = 0x0;
pub const EXIT: u64 = 0x1;

pub const DRAM_OFFSET: u64 = 0x40000000;
pub const STACK_OFFSET: u64 = USER_MEMORY_SIZE as u64;
pub const USER_MEMORY_SIZE: usize = u32::MAX as usize;

#[derive(Debug)]
pub struct Emulator {
    regs: [u64; 32],
    memory: Vec<u8>,
    pc: u64,

    exiting: bool,
    exit_code: i32,

    console: Vec<u8>,
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            pc: 0,
            memory: vec![0; USER_MEMORY_SIZE as usize],
            regs: Default::default(),
            exit_code: 0,
            exiting: false,
            console: Vec::new(),
        }
    }
}

impl Emulator {
    /// Load program data into memory at offset [`Addr`].
    ///
    /// Subsequently sets pc to offset.
    pub fn flash_prgm(&mut self, prgm: &[u8], offset: u32) {
        println!("flashing program...");

        for (byte, b) in self
            .memory_mut(offset, prgm.len())
            .iter_mut()
            .zip(prgm.iter())
        {
            *byte = *b;
        }
        self.pc = offset as u64;

        println!("finished!");
    }

    /// Run program until the exit ecall is made.
    pub fn run(&mut self) {
        loop {
            let raw_instr = self.read_pc();
            if !self.step(raw_instr as u32) {
                return;
            }
        }
    }

    /// Run program until iterations is reached or the exit ecall made.
    #[allow(unused)]
    pub fn run_for(&mut self, mut iterations: usize) {
        loop {
            let raw_instr = self.read_pc();
            if iterations == 0 {
                return;
            }

            if !self.step(raw_instr as u32) {
                return;
            }

            iterations -= 1;
        }
    }

    /// Slice of memory at location [`Addr`].
    ///
    /// User address space: 0..[`u32::MAX`].
    fn memory(&self, offset: u32, len: usize) -> &[u8] {
        let start = offset as usize;
        let end = offset as usize + len;
        // println!(
        //     "memory fetch => start: {start:#x}: end {end:#x} max: {:#x}",
        //     USER_MEMORY_SIZE
        // );

        if end <= USER_MEMORY_SIZE {
            &self.memory[start..end]
        } else {
            panic!("SEGFAULT");
        }
    }

    /// Slice of mutable memory at location [`Addr`].
    ///
    /// User address space: 0..[`u32::MAX`].
    fn memory_mut(&mut self, offset: u32, len: usize) -> &mut [u8] {
        let start = offset as usize;
        let end = offset as usize + len;
        // println!(
        //     "mut memory fetch => start: {start:#x}: end {end:#x} max: {:#x}",
        //     USER_MEMORY_SIZE
        // );

        if end <= USER_MEMORY_SIZE {
            &mut self.memory[start..end]
        } else {
            panic!("segfault");
        }
    }

    fn step(&mut self, raw_instr: u32) -> bool {
        if self.exiting {
            // println!("exiting: {}", self.exit_code);
            return false;
        }

        // println!("fetching instr: {:#x}:{raw_instr:#x}", self.pc);

        let instr = crate::decoding::decode(raw_instr);

        self.execute(instr);
        true
    }

    pub fn set(&mut self, reg: Reg, val: u64) {
        self.regs[reg] = val;
    }

    pub fn set_signed(&mut self, reg: Reg, val: i64) {
        self.regs[reg] = val as u64;
    }

    pub fn reg(&self, reg: Reg) -> u64 {
        self.regs[reg]
    }

    pub fn reg_signed(&self, reg: Reg) -> i64 {
        self.regs[reg] as i64
    }

    pub fn add_pc(&mut self, offset: Imm) {
        self.pc = self.pc.wrapping_add(offset.val());
    }

    pub fn read_pc(&mut self) -> u64 {
        self.load(Offset(Reg::Zero, Imm::Pos(self.pc as u64)), 4)
    }

    pub fn load(&mut self, offset: Offset, bytes: usize) -> u64 {
        let mut val = 0;
        let offset = self.reg(offset.0).wrapping_add(offset.1.val());

        if offset == NULL {
            self.exiting = true;
            self.exit_code = 139;
        } else if offset == EXIT {
            self.exiting = true;
            self.exit_code = 0;
        }

        let memory = self.memory(offset as u32, bytes);

        for (i, byte) in memory.iter().enumerate() {
            val += (*byte as u64) << (i * 8);
        }

        val
    }

    pub fn store(&mut self, offset: Offset, bytes: usize, val: u64) {
        let offset = self.reg(offset.0).wrapping_add(offset.1.val());

        let memory = self.memory_mut(offset as u32, bytes);
        for (i, byte) in memory.iter_mut().enumerate() {
            *byte = (val >> (i * 8)) as u8;
        }

        if offset == CONSOLE_OFFSET {
            self.console.push(self.memory(offset as u32, 1)[0]);
        }
    }

    fn execute(&mut self, instr: Instr) {
        println!("\t\texecuting: {instr:?}");

        self.set(Reg::Zero, 0);

        match instr {
            Instr::Lui(dst, imm) => {
                self.set_signed(dst, ((imm.val_signed() as i64) >> 12) << 12);
            }
            Instr::Auipc(dst, imm) => {
                self.set_signed(
                    dst,
                    self.pc as i64 + (((imm.val_signed() as i64) >> 12) << 12),
                );
            }
            Instr::Addi(dst, src, imm) => {
                self.set_signed(dst, self.reg_signed(src).wrapping_add(imm.val_signed()));
            }
            Instr::Addiw(dst, src, imm) => {
                self.set_signed(dst, self.reg(src).wrapping_add(imm.val()) as i32 as i64);
            }
            Instr::Slti(dst, src, imm) => {
                self.set(
                    dst,
                    if self.reg_signed(src) < imm.val_signed() as i64 {
                        1
                    } else {
                        0
                    },
                );
            }
            Instr::Sltiu(dst, src, imm) => {
                self.set(
                    dst,
                    if self.reg(src) < imm.val() as u64 {
                        1
                    } else {
                        0
                    },
                );
            }
            Instr::Xori(dst, src, imm) => {
                self.set_signed(dst, self.reg_signed(src) ^ (imm.val_signed() as i64));
            }
            Instr::Ori(dst, src, imm) => {
                self.set_signed(dst, self.reg_signed(src) | (imm.val_signed() as i64));
            }
            Instr::Andi(dst, src, imm) => {
                self.set_signed(dst, self.reg_signed(src) & (imm.val_signed() as i64));
            }
            Instr::Slli(dst, src, imm) => {
                self.set_signed(dst, self.reg_signed(src) << (imm.val_signed() & 0b11111));
            }
            Instr::Srli(dst, src, imm) => {
                self.set_signed(dst, self.reg_signed(src) >> (imm.val() & 0b11111));
            }
            Instr::Srai(dst, src, imm) => {
                self.set_signed(dst, self.reg_signed(src) >> (imm.val_signed() & 0b11111));
            }
            Instr::Add(dst, src1, src2) => {
                self.set_signed(
                    dst,
                    self.reg_signed(src1).wrapping_add(self.reg_signed(src2)),
                );
            }
            Instr::Addw(dst, src1, src2) => {
                self.set_signed(
                    dst,
                    self.reg(src1).wrapping_add(self.reg(src2)) as i32 as i64,
                );
            }
            Instr::Sub(dst, src1, src2) => {
                self.set_signed(
                    dst,
                    self.reg_signed(src1).wrapping_sub(self.reg_signed(src2)),
                );
            }
            Instr::Subw(dst, src1, src2) => {
                self.set_signed(
                    dst,
                    self.reg_signed(src1).wrapping_sub(self.reg_signed(src2)) as i32 as i64,
                );
            }
            Instr::Sll(dst, src1, src2) => {
                self.set(dst, self.reg(src1) << (self.reg(src2) & 0b11111));
            }
            Instr::Sllw(dst, src1, src2) => {
                self.set_signed(
                    dst,
                    (self.reg(src1) << (self.reg(src2) & 0b11111)) as i32 as i64,
                );
            }
            Instr::Slt(dst, src1, src2) => {
                self.set(
                    dst,
                    if self.reg_signed(src1) < self.reg_signed(src2) {
                        1
                    } else {
                        0
                    },
                );
            }
            Instr::Sltw(dst, src1, src2) => {
                self.set(
                    dst,
                    if self.reg_signed(src1) < self.reg_signed(src2) {
                        1
                    } else {
                        0
                    },
                );
            }
            Instr::Sltu(dst, src1, src2) => {
                self.set(
                    dst,
                    if self.reg(src1) < self.reg(src2) {
                        1
                    } else {
                        0
                    },
                );
            }
            Instr::Xor(dst, src1, src2) => {
                self.set(dst, self.reg(src1) ^ self.reg(src2));
            }
            Instr::Srl(dst, src1, src2) => {
                self.set(dst, self.reg(src1) >> (self.reg(src2) & 0b11111));
            }
            Instr::Srlw(dst, src1, src2) => {
                self.set_signed(
                    dst,
                    (self.reg(src1) >> (self.reg(src2) & 0b11111)) as i32 as i64,
                );
            }
            Instr::Sra(dst, src1, src2) => {
                self.set_signed(
                    dst,
                    self.reg_signed(src1) >> (self.reg_signed(src2) & 0b11111),
                );
            }
            Instr::Sraw(dst, src1, src2) => {
                self.set_signed(
                    dst,
                    (self.reg_signed(src1) >> (self.reg_signed(src2) & 0b11111)) as i32 as i64,
                );
            }
            Instr::Or(dst, src1, src2) => {
                self.set(dst, self.reg(src1) | self.reg(src2));
            }
            Instr::And(dst, src1, src2) => {
                self.set(dst, self.reg(src1) & self.reg(src2));
            }
            Instr::Lb(dst, offset) => {
                let val = se_byte(self.load(offset, 1) as u8);
                self.set_signed(dst, val);
            }
            Instr::Lh(dst, offset) => {
                let val = se_half(self.load(offset, 2) as u16);
                self.set_signed(dst, val);
            }
            Instr::Ld(dst, offset) => {
                let val = self.load(offset, 8);
                self.set(dst, val);
            }
            Instr::Lbu(dst, offset) => {
                let val = self.load(offset, 1);
                self.set(dst, val);
            }
            Instr::Lhu(dst, offset) => {
                let val = self.load(offset, 2);
                self.set(dst, val);
            }
            Instr::Sb(src, offset) => {
                self.store(offset, 1, self.reg(src));
            }
            Instr::Sh(src, offset) => {
                self.store(offset, 2, self.reg(src));
            }
            Instr::Sw(src, offset) => {
                self.store(offset, 4, self.reg(src));
            }
            Instr::Sd(src, offset) => {
                self.store(offset, 8, self.reg(src));
            }
            Instr::Jal(dst, offset) => {
                self.set(dst, self.pc + 4);
                self.pc = ((self.pc as u32).wrapping_add(offset.val() as u32)) as u64;
            }
            Instr::Jalr(dst, src, offset) => {
                self.set(dst, self.pc + 4);
                let ra = ((self.reg(src) as u32 + offset.val() as u32) >> 1) << 1;
                if ra == 0 {
                    // HACK: main function returns to libc, so unfortunately, it can be assumed
                    // that if the return address is 0, since Reg::Ra will be 0, that we are
                    // returning from main.

                    self.exiting = true;
                    self.exit_code = self.reg(Reg::A(0)) as i32;
                } else {
                    self.pc = (ra.wrapping_add(offset.val() as u32)) as u64;
                }
            }
            Instr::Beq(src1, src2, offset) => {
                if self.reg(src1) == self.reg(src2) {
                    self.add_pc(offset);
                } else {
                    self.add_pc(Imm::new(4));
                }
            }
            Instr::Bne(src1, src2, offset) => {
                if self.reg(src1) != self.reg(src2) {
                    self.add_pc(offset);
                } else {
                    self.add_pc(Imm::new(4));
                }
            }
            Instr::Blt(src1, src2, offset) => {
                if self.reg_signed(src1) < self.reg_signed(src2) {
                    self.add_pc(offset);
                } else {
                    self.add_pc(Imm::new(4));
                }
            }
            Instr::Bge(src1, src2, offset) => {
                if self.reg_signed(src1) >= self.reg_signed(src2) {
                    self.add_pc(offset);
                } else {
                    self.add_pc(Imm::new(4));
                }
            }
            Instr::Bltu(src1, src2, offset) => {
                if self.reg(src1) < self.reg(src2) {
                    self.add_pc(offset);
                } else {
                    self.add_pc(Imm::new(4));
                }
            }
            Instr::Bgeu(src1, src2, offset) => {
                if self.reg(src1) >= self.reg(src2) {
                    self.add_pc(offset);
                } else {
                    self.add_pc(Imm::new(4));
                }
            }
            Instr::Ecall => {
                let syscall = self.reg(Reg::A(7));
                match syscall {
                    // Exit
                    93 => {
                        self.exiting = true;
                        self.exit_code = self.reg_signed(Reg::A(0)) as i32;
                    }
                    // Write
                    64 => match self.reg(Reg::A(0)) {
                        1 => {
                            let buf_addr = self.reg(Reg::A(1)) as usize;
                            let buf_len = self.reg(Reg::A(2)) as usize;
                            // println!("writing {} bytes of buf {} to console.", buf_len, buf_addr);
                            let memory = self.memory(buf_addr as u32, buf_len).to_vec();
                            self.console.extend_from_slice(&memory);
                        }
                        _ => unimplemented!(),
                    },
                    val => println!("invalid syscall: {}", val),
                }
            }

            Instr::Lw(dst, offset) => {
                let val = se_word(self.load(offset, 4) as u32);
                self.set_signed(dst, val);
            }
            Instr::Lwu(dst, offset) => {
                let val = self.load(offset, 4) as u64;
                self.set(dst, val);
            }
            Instr::Slliw(dst, rs1, imm) => {
                let val = self.reg(rs1) as u32;
                let result = val.wrapping_shl(imm.val() as u32);
                self.set_signed(dst, result as i32 as i64);
            }
            Instr::Srliw(dst, rs1, imm) => {
                let val = self.reg(rs1) as u32;
                let result = val.wrapping_shr(imm.val() as u32);
                self.set_signed(dst, result as i32 as i64);
            }
            Instr::Sraiw(dst, rs1, imm) => {
                let val = self.reg(rs1) as i32;
                let result = val.wrapping_shr(imm.val() as u32);
                self.set_signed(dst, result as i64);
            }

            Instr::Mul(dst, rs1, rs2) => {
                self.set(dst, (self.reg(rs1) as u32 * self.reg(rs2) as u32) as u64);
            }
            Instr::Div(dst, rs1, rs2) => {
                self.set(dst, (self.reg(rs1) as u32 / self.reg(rs2) as u32) as u64);
            }
            Instr::Rem(dst, rs1, rs2) => {
                self.set(dst, (self.reg(rs1) as u32 % self.reg(rs2) as u32) as u64);
            }

            Instr::Mulw(dst, rs1, rs2) => {
                self.set(dst, self.reg(rs1) * self.reg(rs2));
            }
            Instr::Divw(dst, rs1, rs2) => {
                self.set(dst, self.reg(rs1) / self.reg(rs2));
            }
            Instr::Remw(dst, rs1, rs2) => {
                self.set(dst, self.reg(rs1) % self.reg(rs2));
            }
        }

        match instr {
            Instr::Bgeu(_, _, _)
            | Instr::Bltu(_, _, _)
            | Instr::Bge(_, _, _)
            | Instr::Blt(_, _, _)
            | Instr::Bne(_, _, _)
            | Instr::Beq(_, _, _)
            | Instr::Jal(_, _)
            | Instr::Jalr(_, _, _) => {}
            _ => {
                self.add_pc(Imm::new(4));
            }
        }

        self.set(Reg::Zero, 0);
    }
}

fn se_byte(byte: u8) -> i64 {
    ((byte as i64) << 56) >> 56
}

fn se_half(byte: u16) -> i64 {
    ((byte as i64) << 48) >> 48
}

fn se_word(byte: u32) -> i64 {
    ((byte as i64) << 32) >> 32
}

pub fn run_emulator(prgm: &[u8]) -> Emulator {
    let mut emulator = Emulator::default();
    emulator.flash_prgm(prgm, DRAM_OFFSET as u32);
    emulator.set(Reg::Sp, STACK_OFFSET);
    emulator.run();
    emulator
}

pub fn print_emulator(emulator: &Emulator) {
    println!("Registers:");
    for i in 0..32 {
        println!("x{} \t{:#018x}", i, emulator.regs[i]);
    }

    println!("\nHeap:");
    for mem in 0..8 {
        print!("{:#09x}\t", mem * 8 * 16 + HEAP_OFFSET as usize);
        let index = mem * 16 + HEAP_OFFSET as usize;
        for byte in emulator.memory[index..index + 16].iter() {
            print!("{:02X} ", byte);
        }
        println!();
    }

    println!("\nConsole:\n{}", String::from_utf8_lossy(&emulator.console));
    // println!("\nexit code: {}", emulator.exit_code);
}

/// https://github.com/d0iasm/rvemu/blob/main/tests/rv32i.rs
#[cfg(test)]
mod tests {
    use super::*;

    const REGISTERS_COUNT: usize = 32;

    /// Create registers for x0-x31 with expected values.
    pub fn create_xregs(non_zero_regs: Vec<(usize, u64)>) -> [u64; REGISTERS_COUNT] {
        let mut xregs = [0; REGISTERS_COUNT];

        // Based on XRegisters::new().
        // xregs[2] = DEFAULT_SP;
        xregs[2] = 0;
        // xregs[11] = POINTER_TO_DTB;

        for pair in non_zero_regs.iter() {
            xregs[pair.0] = pair.1;
        }
        xregs
    }

    /// Start a test and check if the registers are expected.
    pub fn run(emulator: &mut Emulator, data: Vec<u8>, expected_xregs: &[u64; 32]) {
        emulator.flash_prgm(&data, 0);
        emulator.run_for(data.len() / 4);

        for (i, e) in expected_xregs.iter().enumerate() {
            if *e != 0 {
                assert_eq!(*e, emulator.regs[i], "fails at {}", i);
            }
        }
    }

    #[test]
    fn lb_rd_offset_rs1() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x50, 0x00, // addi x16, x0, 5
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x03, 0x09, 0x40, 0x00, // lb x18, 4(x0)
        ];
        let expected_xregs = create_xregs(vec![(16, 5), (17, 3), (18, 0x93)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn lh_rd_offset_rs1() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x50, 0x00, // addi x16, x0, 5
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x03, 0x19, 0x40, 0x00, // lh x18, 4(x0)
        ];
        let expected_xregs = create_xregs(vec![(16, 5), (17, 3), (18, 0x0893)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn lw_rd_offset_rs1() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x50, 0x00, // addi x16, x0, 5
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x03, 0x29, 0x40, 0x00, // lw x18, 4(x0)
        ];
        let expected_xregs = create_xregs(vec![(16, 5), (17, 3), (18, 0x300893)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn lbu_rd_offset_rs1() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x50, 0x00, // addi x16, x0, 5
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x03, 0x49, 0x40, 0x00, // lbu x18, 4(x0)
        ];
        let expected_xregs = create_xregs(vec![(16, 5), (17, 3), (18, 0x93)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn lhu_rd_offset_rs1() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x50, 0x00, // addi x16, x0, 5
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x03, 0x59, 0x40, 0x00, // lbu x18, 4(x0)
        ];
        let expected_xregs = create_xregs(vec![(16, 5), (17, 3), (18, 0x0893)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn addi_rd_rs1_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x93, 0x0f, 0x40, 0x00, // addi x31, x0, 4
        ];
        let expected_xregs = create_xregs(vec![(31, 4)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn slli_rd_rs1_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x20, 0x00, // addi x16 x0, 2
            0x93, 0x18, 0x38, 0x00, // slli x17, x16, 3
        ];
        let expected_xregs = create_xregs(vec![(16, 2), (17, 16)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn slti_rd_rs1_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0xb0, 0xff, // addi x16 x0, -5
            0x93, 0x28, 0xe8, 0xff, // slti x17, x16, -2
        ];
        let expected_xregs = create_xregs(vec![(16, -5i64 as u64), (17, 1)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn sltiu_rd_rs1_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x20, 0x00, // addi x16, x0, 2
            0x93, 0x38, 0x58, 0x00, // sltiu, x17, x16, 5
        ];
        let expected_xregs = create_xregs(vec![(16, 2), (17, 1)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn xori_rd_rs1_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x48, 0x68, 0x00, // xori, x17, x16, 6
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 5)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn srai_rd_rs1_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x80, 0xff, // addi x16, x0, -8
            0x93, 0x58, 0x28, 0x40, // srai x17, x16, 2
        ];
        let expected_xregs = create_xregs(vec![(16, -8i64 as u64), (17, -2i64 as u64)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn srli_rd_rs1_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x80, 0x00, // addi x16, x0, 8
            0x93, 0x58, 0x28, 0x00, // srli x17, x16, 2
        ];
        let expected_xregs = create_xregs(vec![(16, 8), (17, 2)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn ori_rd_rs1_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x68, 0x68, 0x00, // ori, x17, x16, 6
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 7)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn andi_rd_rs1_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x40, 0x00, // addi x16, x0, 4
            0x93, 0x78, 0x78, 0x00, // andi, x17, x16, 7
        ];
        let expected_xregs = create_xregs(vec![(16, 4), (17, 4)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn auipc_rd_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x17, 0x28, 0x00, 0x00, // auipc x16, 2
        ];
        let expected_xregs = create_xregs(vec![(16, 0x2000)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn sb_rs2_offset_rs1() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0xb0, 0xff, // addi x16, x0, -5
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x23, 0x02, 0x00, 0x01, // sb x16, 4(x0)
            0x03, 0x09, 0x40, 0x00, // lb x18, 4(x0)
        ];
        let expected_xregs = create_xregs(vec![
            (16, -5i64 as u64),
            (17, 3),
            (18, ((-5i64 as u64) << 56) >> 56),
        ]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn sh_rs2_offset_rs1() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x00, 0xc0, // addi x16, x0, -1024
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x23, 0x12, 0x00, 0x01, // sh x16, 4(x0)
            0x03, 0x19, 0x40, 0x00, // lh x18, 4(x0)
        ];
        let expected_xregs = create_xregs(vec![
            (16, -1024i64 as u64),
            (17, 3),
            (18, ((-1024i64 as u64) << 48) >> 48),
        ]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn sw_rs2_offset_rs1() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x00, 0x80, // addi x16, x0, -2048
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x23, 0x22, 0x00, 0x01, // sw x16, 4(x0)
            0x03, 0x29, 0x40, 0x00, // lw x18, 4(x0)
        ];
        let expected_xregs = create_xregs(vec![
            (16, -2048i64 as u64),
            (17, 3),
            (18, ((-2048i64 as u64) << 32) >> 32),
        ]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn add_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x93, 0x01, 0x50, 0x00, // addi x3, x0, 5
            0x13, 0x02, 0x60, 0x00, // addi x4, x0, 6
            0x33, 0x81, 0x41, 0x00, // add x2, x3, x4
        ];
        let expected_xregs = create_xregs(vec![(2, 11), (3, 5), (4, 6)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn sub_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x93, 0x01, 0x50, 0x00, // addi x3, x0, 5
            0x13, 0x02, 0x60, 0x00, // addi x4, x0, 6
            0x33, 0x81, 0x41, 0x40, // sub x2, x3, x4
        ];
        let expected_xregs = create_xregs(vec![(2, -1i64 as u64), (3, 5), (4, 6)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn sll_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x80, 0x00, // addi x16, x0, 8
            0x33, 0x19, 0x18, 0x01, // sll x18, x16, x17
        ];
        let expected_xregs = create_xregs(vec![(16, 8), (17, 2), (18, 32)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn slt_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x80, 0xff, // addi x16, x0, -8
            0x93, 0x08, 0x20, 0x00, // addi x17, x0, 2
            0x33, 0x29, 0x18, 0x01, // slt x18, x16, x17
        ];
        let expected_xregs = create_xregs(vec![(16, -8i64 as u64), (17, 2), (18, 1)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn sltu_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x80, 0x00, // addi x16, x0, 8
            0x93, 0x08, 0x20, 0x00, // addi x17, x0, 2
            0x33, 0xb9, 0x08, 0x01, // slt x18, x17, x16
        ];
        let expected_xregs = create_xregs(vec![(16, 8), (17, 2), (18, 1)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn xor_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x08, 0x60, 0x00, // addi x17, x0, 6
            0x33, 0x49, 0x18, 0x01, // xor x18, x16, x17
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 6), (18, 5)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn srl_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x00, 0x01, // addi x16, x0, 16
            0x93, 0x08, 0x20, 0x00, // addi x17, x0, 2
            0x33, 0x59, 0x18, 0x01, // srl x18, x16, x17
        ];
        let expected_xregs = create_xregs(vec![(16, 16), (17, 2), (18, 4)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn sra_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x00, 0xff, // addi x16, x0, -16
            0x93, 0x08, 0x20, 0x00, // addi x17, x0, 2
            0x33, 0x59, 0x18, 0x41, // sra x18, x16, x17
        ];
        let expected_xregs = create_xregs(vec![(16, -16i64 as u64), (17, 2), (18, -4i64 as u64)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn or_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x08, 0x50, 0x00, // addi x17, x0, 5
            0x33, 0x69, 0x18, 0x01, // or x18, x16, x17
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 5), (18, 7)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn and_rd_rs1_rs2() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x08, 0x50, 0x00, // addi x17, x0, 5
            0x33, 0x79, 0x18, 0x01, // and x18, x16, x17
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 5), (18, 1)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn lui_rd_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x37, 0x28, 0x00, 0x00, // lui x16, 2
        ];
        let expected_xregs = create_xregs(vec![(16, 8192)]);

        run(&mut emu, data, &expected_xregs);
    }

    #[test]
    fn beq_rs1_rs2_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x63, 0x06, 0x18, 0x01, // beq x16, x17, 12
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 3)]);

        run(&mut emu, data, &expected_xregs);

        assert_eq!(20, emu.pc);
    }

    #[test]
    fn bne_rs1_rs2_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x08, 0x50, 0x00, // addi x17, x0, 5
            0x63, 0x16, 0x18, 0x01, // bne x16, x17, 12
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 5)]);

        run(&mut emu, data, &expected_xregs);

        assert_eq!(20, emu.pc);
    }

    #[test]
    fn blt_rs1_rs2_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0xd0, 0xff, // addi x16, x0, -3
            0x93, 0x08, 0x50, 0x00, // addi x17, x0, 5
            0x63, 0x46, 0x18, 0x01, // blt x16, x17, 12
        ];
        let expected_xregs = create_xregs(vec![(16, -3i64 as u64), (17, 5)]);

        run(&mut emu, data, &expected_xregs);

        assert_eq!(20, emu.pc);
    }

    #[test]
    fn bge_rs1_rs2_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0xd0, 0xff, // addi x16, x0, -3
            0x93, 0x08, 0xd0, 0xff, // addi x17, x0, -3
            0x63, 0x56, 0x18, 0x01, // bge x16, x17, 12
        ];
        let expected_xregs = create_xregs(vec![(16, -3i64 as u64), (17, -3i64 as u64)]);

        run(&mut emu, data, &expected_xregs);

        assert_eq!(20, emu.pc);
    }

    #[test]
    fn bltu_rs1_rs2_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x08, 0x50, 0x00, // addi x17, x0, 5
            0x63, 0x66, 0x18, 0x01, // bltu x16, x17, 12
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 5)]);

        run(&mut emu, data, &expected_xregs);

        assert_eq!(20, emu.pc);
    }

    #[test]
    fn bgeu_rs1_rs2_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x50, 0x00, // addi x16, x0, 5
            0x93, 0x08, 0x30, 0x00, // addi x17, x0, 3
            0x63, 0x76, 0x18, 0x01, // bgeu x16, x17, 12
        ];
        let expected_xregs = create_xregs(vec![(16, 5), (17, 3)]);

        run(&mut emu, data, &expected_xregs);

        assert_eq!(20, emu.pc);
    }

    #[test]
    fn jalr_rd_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x08, 0x50, 0x00, // addi x17, x0, 5
            0x67, 0x09, 0xc0, 0x02, // jalr x18, x0, 44
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 5), (18, 12)]);

        run(&mut emu, data, &expected_xregs);

        assert_eq!(44, emu.pc);
    }

    #[test]
    fn jal_rd_imm() {
        let mut emu = Emulator::default();

        let data = vec![
            0x13, 0x08, 0x30, 0x00, // addi x16, x0, 3
            0x93, 0x08, 0x50, 0x00, // addi x17, x0, 5
            0x6f, 0x09, 0xc0, 0x00, // jal x18, 12
        ];
        let expected_xregs = create_xregs(vec![(16, 3), (17, 5), (18, 12)]);

        run(&mut emu, data, &expected_xregs);

        assert_eq!(20, emu.pc);
    }
}
