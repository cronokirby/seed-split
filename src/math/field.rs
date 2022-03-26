use std::ops;

#[derive(Clone, Copy, Debug)]
// Only implement equality for tests. This is to avoid the temptation to introduce
// a timing leak through equality comparison.
#[cfg_attr(test, derive(PartialEq))]
pub struct BPoly<const N: usize> {
    data: [u64; N],
}

impl<const N: usize> BPoly<N> {
    pub fn zero() -> Self {
        Self { data: [0; N] }
    }

    pub fn one() -> Self {
        // Not sure if there's a more elegant way to write this
        let mut data = [0; N];
        data[0] = 1;
        Self { data }
    }
}

impl<const N: usize> ops::AddAssign for BPoly<N> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.data[i] ^= rhs.data[i]
        }
    }
}

impl<const N: usize> ops::Add for BPoly<N> {
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
        fn arb_bpoly()(data in any::<[u64;4]>()) -> BPoly<4> {
            BPoly { data }
        }
    }

    proptest! {
        #[test]
        fn test_addition_commutative(a in arb_bpoly(), b in arb_bpoly()) {
            assert_eq!(a + b, b + a)
        }
    }

    proptest! {
        #[test]
        fn test_addition_associative(a in arb_bpoly(), b in arb_bpoly(), c in arb_bpoly()) {
            assert_eq!(a + (b + c), (a + b) + c);
        }
    }

    proptest! {
        #[test]
        fn test_add_zero_identity(a in arb_bpoly()) {
            let zero = BPoly::zero();
            assert_eq!(a + zero, a);
            assert_eq!(zero + a, a);
        }
    }

    #[test]
    fn test_one_plus_one_is_zero() {
        let one = BPoly::<4>::one();
        assert_eq!(one + one, BPoly::zero())
    }
}
