use std::ops;

const CHUNKS: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct GF256 {
    data: [u64; CHUNKS],
}

impl ops::AddAssign for GF256 {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..CHUNKS {
            self.data[i] ^= rhs.data[i]
        }
    }
}

impl ops::Add for GF256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut out = self;
        out += rhs;
        out
    }
}
