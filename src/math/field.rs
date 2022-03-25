use std::ops;

const CHUNKS: usize = 4;

#[derive(Clone, Copy, Debug)]
// Only implement equality for tests. This is to avoid the temptation to introduce
// a timing leak through equality comparison.
#[cfg_attr(test, derive(PartialEq))]
pub struct GF256 {
    data: [u64; CHUNKS],
}

impl GF256 {
    pub fn zero() -> Self {
        Self { data: [0; CHUNKS] }
    }

    pub fn one() -> Self {
        Self { data: [1, 0, 0, 0] }
    }
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

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    // We can generate an arbitrary element just by choosing random bits
    prop_compose! {
        fn arb_gf256()(data in any::<[u64;4]>()) -> GF256 {
            GF256 { data }
        }
    }

    proptest! {
        #[test]
        fn test_addition_commutative(a in arb_gf256(), b in arb_gf256()) {
            assert_eq!(a + b, b + a)
        }
    }

    proptest! {
        #[test]
        fn test_addition_associative(a in arb_gf256(), b in arb_gf256(), c in arb_gf256()) {
            assert_eq!(a + (b + c), (a + b) + c);
        }
    }

    proptest! {
        #[test]
        fn test_add_zero_identity(a in arb_gf256()) {
            let zero = GF256::zero();
            assert_eq!(a + zero, a);
            assert_eq!(zero + a, a);
        }
    }

    #[test]
    fn test_one_plus_one_is_zero() {
        let one = GF256::one();
        assert_eq!(one + one, GF256::zero())
    }
}
