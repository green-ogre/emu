use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Copy)]
pub enum Reg {
    Zero,
    Ra,
    Sp,
    Gp,
    Tp,
    T(usize),
    S(usize),
    A(usize),
}

impl Reg {
    pub fn new(reg: u32) -> Self {
        match reg {
            0 => Self::Zero,
            1 => Self::Ra,
            2 => Self::Sp,
            3 => Self::Gp,
            4 => Self::Tp,
            5 => Self::T(0),
            6 => Self::T(1),
            7 => Self::T(2),
            8 => Self::S(0),
            9 => Self::S(1),
            10 => Self::A(0),
            11 => Self::A(1),
            12 => Self::A(2),
            13 => Self::A(3),
            14 => Self::A(4),
            15 => Self::A(5),
            16 => Self::A(6),
            17 => Self::A(7),

            18 => Self::S(2),
            19 => Self::S(3),
            20 => Self::S(4),
            21 => Self::S(5),
            22 => Self::S(6),
            23 => Self::S(7),
            24 => Self::S(8),
            25 => Self::S(9),
            26 => Self::S(10),
            27 => Self::S(11),

            28 => Self::T(3),
            29 => Self::T(4),
            30 => Self::T(5),
            31 => Self::T(6),

            reg => panic!("invalid register: x{reg}"),
        }
    }

    pub fn reg_index(&self) -> usize {
        match self {
            Reg::Zero => 0,
            Reg::Ra => 1,
            Reg::Sp => 2,
            Reg::Gp => 3,
            Reg::Tp => 4,
            Reg::T(i) => match i {
                0 => 5,
                1 => 6,
                2 => 7,
                3 => 28,
                4 => 29,
                5 => 30,
                6 => 31,
                _ => panic!("invalid t register: {i}"),
            },
            Reg::S(i) => match i {
                0 => 8,
                1 => 9,
                2 => 18,
                3 => 19,
                4 => 20,
                5 => 21,
                6 => 22,
                7 => 23,
                8 => 24,
                9 => 25,
                10 => 26,
                11 => 27,
                _ => panic!("invalid t register: {i}"),
            },
            Reg::A(i) => match i {
                0 => 10,
                1 => 11,
                2 => 12,
                3 => 13,
                4 => 14,
                5 => 15,
                6 => 16,
                7 => 17,
                _ => panic!("invalid t register: {i}"),
            },
        }
    }
}

impl Index<Reg> for [u64; 32] {
    type Output = u64;
    fn index(&self, index: Reg) -> &Self::Output {
        &self[index.reg_index()]
    }
}

impl IndexMut<Reg> for [u64; 32] {
    fn index_mut(&mut self, index: Reg) -> &mut Self::Output {
        &mut self[index.reg_index()]
    }
}

#[derive(Clone, Copy)]
pub enum Imm {
    Pos(u64),
    Neg(u64),
}

impl Debug for Imm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pos(val) => f.debug_tuple("Imm").field(val).finish(),
            Self::Neg(val) => f.debug_tuple("Imm").field(&(*val as i64)).finish(),
        }
    }
}

impl Imm {
    pub const ZERO: Self = Self::Pos(0);

    pub fn new(imm: i64) -> Self {
        if imm < 0 {
            Self::Neg(imm as u64)
        } else {
            Self::Pos(imm as u64)
        }
    }

    pub fn val(&self) -> u64 {
        *match self {
            Self::Pos(val) => val,
            Self::Neg(val) => val,
        }
    }

    pub fn val_signed(&self) -> i64 {
        match self {
            Self::Pos(val) => *val as i64,
            Self::Neg(val) => *val as i64,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Offset(pub Reg, pub Imm);
