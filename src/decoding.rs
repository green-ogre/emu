use crate::instruction_set::*;
use crate::primitives::*;

pub fn decode(raw_instr: u32) -> Instr {
    let opcode = raw_instr & 0b1111111;
    match opcode {
        // R-type
        0x33 | 0b111011 => {
            let rd = (raw_instr >> 7) & 0b11111;
            let fn3 = (raw_instr >> 12) & 0b111;
            let rs1 = (raw_instr >> 15) & 0b11111;
            let rs2 = (raw_instr >> 20) & 0b11111;
            let fn7 = raw_instr >> 25;
            // println!("decoded instr: {fn7:#x} {rs2:#x} {rs1:#x} {fn3:#x} {rd:#x} {opcode:#x}");

            let rd = Reg::new(rd);
            let rs1 = Reg::new(rs1);
            let rs2 = Reg::new(rs2);

            match opcode {
                0x33 => match fn7 {
                    0b0000000 => match fn3 {
                        0b000 => Instr::Add(rd, rs1, rs2),
                        0b001 => Instr::Sll(rd, rs1, rs2),
                        0b010 => Instr::Slt(rd, rs1, rs2),
                        0b011 => Instr::Sltu(rd, rs1, rs2),
                        0b100 => Instr::Xor(rd, rs1, rs2),
                        0b101 => Instr::Srl(rd, rs1, rs2),
                        0b110 => Instr::Or(rd, rs1, rs2),
                        0b111 => Instr::And(rd, rs1, rs2),
                        val => panic!("{val:#b}"),
                    },
                    0b0100000 => match fn3 {
                        0b000 => Instr::Sub(rd, rs1, rs2),
                        0b101 => Instr::Sra(rd, rs1, rs2),
                        val => panic!("{val:#b}"),
                    },
                    0b0000001 => match fn3 {
                        0b000 => Instr::Mul(rd, rs1, rs2),
                        0b100 => Instr::Div(rd, rs1, rs2),
                        0b110 => Instr::Rem(rd, rs1, rs2),
                        val => panic!("{val:#b}"),
                    },
                    val => panic!("{val:#b}"),
                },
                0b111011 => match fn7 {
                    0b000 => match fn3 {
                        0b000 => Instr::Addw(rd, rs1, rs2),
                        0b001 => Instr::Sllw(rd, rs1, rs2),
                        0b101 => Instr::Srlw(rd, rs1, rs2),
                        val => panic!("{val:#b}"),
                    },
                    0b0100000 => match fn3 {
                        0b000 => Instr::Subw(rd, rs1, rs2),
                        0b101 => Instr::Sraw(rd, rs1, rs2),
                        val => panic!("{val:#b}"),
                    },
                    0b0000001 => match fn3 {
                        0b000 => Instr::Mulw(rd, rs1, rs2),
                        0b100 => Instr::Divw(rd, rs1, rs2),
                        0b110 => Instr::Remw(rd, rs1, rs2),
                        val => panic!("{val:#b}"),
                    },
                    val => panic!("{val:#b}"),
                },
                val => panic!("{val:#b}"),
            }
        }
        // I-type
        0x03 | 0x67 | 0x13 | 0b0011011 => {
            let rd = (raw_instr >> 7) & 0b11111;
            let fn3 = (raw_instr >> 12) & 0b111;
            let rs1 = (raw_instr >> 15) & 0b11111;
            let imm = ((raw_instr as i32) >> 20) as i64;

            // let shamt = (raw_instr >> 20) & 0b11111;
            let shamt = (raw_instr >> 20) & 0x3f; // Changed to 6 bits for RV64I
            let fn7 = (raw_instr >> 25) & 0b1111111;

            let imm = Imm::new(imm);
            let shamt = Imm::Pos(shamt as u64);

            // println!("decoded instr: {imm:?} {rs1:#x} {fn3:#x} {rd:#x} {opcode:#x}");

            let rd = Reg::new(rd);
            let rs1 = Reg::new(rs1);

            match opcode {
                0x03 => match fn3 {
                    0b000 => Instr::Lb(rd, Offset(rs1, imm)),
                    0b001 => Instr::Lh(rd, Offset(rs1, imm)),
                    0b010 => Instr::Lw(rd, Offset(rs1, imm)),
                    0b011 => Instr::Ld(rd, Offset(rs1, imm)),
                    0b100 => Instr::Lbu(rd, Offset(rs1, imm)),
                    0b101 => Instr::Lhu(rd, Offset(rs1, imm)),
                    0b110 => Instr::Lwu(rd, Offset(rs1, imm)),
                    _ => panic!("Invalid fn3 for LOAD: {fn3:#b}"),
                },
                0x13 => match fn3 {
                    0b000 => Instr::Addi(rd, rs1, imm),
                    0b010 => Instr::Slti(rd, rs1, imm),
                    0b011 => Instr::Sltiu(rd, rs1, imm),
                    0b100 => Instr::Xori(rd, rs1, imm),
                    0b110 => Instr::Ori(rd, rs1, imm),
                    0b111 => Instr::Andi(rd, rs1, imm),
                    0b001 => Instr::Slli(rd, rs1, shamt),
                    0b101 => match fn7 >> 1 {
                        0b000000 => Instr::Srli(rd, rs1, shamt),
                        0b010000 => Instr::Srai(rd, rs1, shamt),
                        _ => panic!("Invalid fn7 for SRLI/SRAI: {fn7:#b}"),
                    },
                    _ => panic!("Invalid fn3 for OP-IMM: {fn3:#b}"),
                },
                0x67 => match fn3 {
                    0b000 => Instr::Jalr(rd, rs1, imm),
                    _ => panic!("Invalid fn3 for JALR: {fn3:#b}"),
                },
                0x1b => match fn3 {
                    0b000 => Instr::Addiw(rd, rs1, imm),
                    0b001 => Instr::Slliw(rd, rs1, shamt),
                    0b101 => match fn7 {
                        0b0000000 => Instr::Srliw(rd, rs1, shamt),
                        0b0100000 => Instr::Sraiw(rd, rs1, shamt),
                        _ => panic!("Invalid fn7 for SRLIW/SRAIW: {fn7:#b}"),
                    },
                    _ => panic!("Invalid fn3 for OP-IMM-32: {fn3:#b}"),
                },
                _ => unreachable!(),
            }
        }
        // S-type
        0x23 => {
            let fn3 = (raw_instr >> 12) & 0b111;
            let rs1 = (raw_instr >> 15) & 0b11111;
            let rs2 = (raw_instr >> 20) & 0b11111;

            let imm_11_5 = (raw_instr >> 25) & 0x7F;
            let imm_4_0 = (raw_instr >> 7) & 0x1F;

            let imm_12bit = (imm_11_5 << 5) | imm_4_0;

            let imm_64 = ((imm_12bit as i64) << 52) >> 52;
            let imm = if imm_64 < 0 {
                Imm::Neg(imm_64 as u64)
            } else {
                Imm::Pos(imm_64 as u64)
            };

            // println!("decoded instr: {imm:?} {rs2:#x} {rs1:#x} {fn3:#x} {opcode:#x}");

            let rs1 = Reg::new(rs1);
            let rs2 = Reg::new(rs2);
            let offset = Offset(rs1, imm);

            match fn3 {
                0b000 => Instr::Sb(rs2, offset),
                0b001 => Instr::Sh(rs2, offset),
                0b010 => Instr::Sw(rs2, offset),
                0b011 => Instr::Sd(rs2, offset),
                val => panic!("{val:#b}"),
            }
        }
        // B-type
        0x63 => {
            let fn3 = (raw_instr >> 12) & 0b111;
            let rs1 = (raw_instr >> 15) & 0b11111;
            let rs2 = (raw_instr >> 20) & 0b11111;

            let mut imm: i64 = 0;
            let raw_instr = raw_instr as i64;
            imm |= ((raw_instr >> 31) & 0x1) << 12; // imm[12]
            imm |= ((raw_instr >> 7) & 0x1) << 11; // imm[11]
            imm |= ((raw_instr >> 25) & 0x3f) << 5; // imm[10:5]
            imm |= ((raw_instr >> 8) & 0xf) << 1; // imm[4:1]

            // Sign extend
            if (imm & 0x1000) > 0 {
                imm = ((imm as u64) | 0xFFFFFFFFFFFFE000) as i64;
            }
            let imm = Imm::new(imm);

            // println!("decoded instr: {imm:?} {rs2:#x} {rs1:#x} {fn3:#x} {opcode:#x}");

            let rs1 = Reg::new(rs1);
            let rs2 = Reg::new(rs2);

            match fn3 {
                0b000 => Instr::Beq(rs1, rs2, imm),
                0b001 => Instr::Bne(rs1, rs2, imm),
                0b100 => Instr::Blt(rs1, rs2, imm),
                0b101 => Instr::Bge(rs1, rs2, imm),
                0b110 => Instr::Bltu(rs1, rs2, imm),
                0b111 => Instr::Bgeu(rs1, rs2, imm),
                val => panic!("{val:#b}"),
            }
        }
        // U-type
        0x37 | 0x17 => {
            let rd = (raw_instr >> 7) & 0b11111;
            let imm = (raw_instr >> 12) << 12;

            let imm = if imm & 0x800 != 0 {
                Imm::Neg((imm as u64) | 0xFFFFFFFFFFFFF000u64)
            } else {
                Imm::Pos(imm as u64)
            };

            // println!("decoded instr: {imm:?} {rd:#x} {opcode:#x}");

            let rd = Reg::new(rd);

            match opcode {
                0b0110111 => Instr::Lui(rd, imm),
                0b0010111 => Instr::Auipc(rd, imm),
                val => panic!("{val:#b}"),
            }
        }
        // J-type
        0x6F => {
            let rd = (raw_instr >> 7) & 0b11111;

            let raw_instr = raw_instr as i64;
            let mut imm = 0;
            imm |= ((raw_instr >> 31) & 0x1) << 20; // imm[20]
            imm |= ((raw_instr >> 12) & 0xFF) << 12; // imm[19:12]
            imm |= ((raw_instr >> 20) & 0x1) << 11; // imm[11]
            imm |= ((raw_instr >> 21) & 0x3FF) << 1; // imm[10:1]

            if (imm & 0x100000) > 0 {
                imm = ((imm as u64) | 0xFFFFFFFFFFF00000) as i64
            }

            let imm = Imm::new(imm);

            // println!("decoded instr: {imm:?} {rd:#x} {opcode:#x}");

            let rd = Reg::new(rd);

            Instr::Jal(rd, imm)
        }
        0b1110011 => Instr::Ecall,
        opcode => panic!("invalid opcode: {:#x}", opcode),
    }
}
