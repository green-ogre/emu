#[derive(Debug, Clone, Copy)]
pub enum Button {
    Zero,
    One,
    Two,
    Three,
}

impl Button {
    pub fn new(b: usize) -> Self {
        match b {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            _ => panic!("{b}"),
        }
    }
}
