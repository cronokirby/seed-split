use std::ops;

/// Represents some kind of field.
///
/// We require addition and multiplication, along with inversion.
///
/// We require copy mainly for convenience.
pub trait Field:
    Copy + ops::Add + ops::AddAssign + ops::Sub + ops::SubAssign + ops::Neg + ops::Mul + ops::MulAssign
{
    /// Return the multiplicative inverse of this element.
    fn invert(self) -> Self;
    /// Return the multlicative unit in this field.
    fn one() -> Self;
    /// Return the additive identity in the field.
    fn zero() -> Self;
}

#[derive(Clone, Copy, Debug)]
// Only implement equality for tests. This is to avoid the temptation to introduce
// a timing leak through equality comparison.
#[cfg_attr(test, derive(PartialEq))]
struct BPoly<const N: usize> {
    data: [u64; N],
}

impl<const N: usize> BPoly<N> {
    fn zero() -> Self {
        Self { data: [0; N] }
    }

    fn one() -> Self {
        // Not sure if there's a more elegant way to write this
        let mut data = [0; N];
        data[0] = 1;
        Self { data }
    }

    fn times_z(&mut self, start: u64) -> u64 {
        let mut top = start;
        for x in &mut self.data {
            let new_top = *x >> 63;
            *x = (*x << 1) | top;
            top = new_top;
        }
        top
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

impl<const N: usize> ops::Mul for BPoly<N> {
    type Output = (Self, Self);

    fn mul(self, rhs: Self) -> Self::Output {
        // Algorithm 2.35 in "Guide to Elliptic Curve Cryptography"
        let (mut out_hi, mut out_lo) = (Self::zero(), Self::zero());

        for k in (0..64).rev() {
            for j in 0..N {
                if ((self.data[j] >> k) & 1) != 1 {
                    continue;
                }
                // Hopefully all of this can get inlined
                let mut view = Self::zero();
                view.data[..(N - j)].copy_from_slice(&out_lo.data[j..]);
                view.data[(N - j)..].copy_from_slice(&out_hi.data[..j]);
                view += rhs;
                out_lo.data[j..].copy_from_slice(&view.data[..(N - j)]);
                out_hi.data[..j].copy_from_slice(&view.data[(N - j)..]);
            }
            if k != 0 {
                let top = out_lo.times_z(0);
                out_hi.times_z(top);
            }
        }
        (out_hi, out_lo)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GF128(BPoly<2>);

impl ops::Add for GF128 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut out = self;
        out += rhs;
        out
    }
}

impl ops::AddAssign for GF128 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl ops::Neg for GF128 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self
    }
}

impl ops::Sub for GF128 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl ops::SubAssign for GF128 {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
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

    proptest! {
        #[test]
        fn test_multiplication_commutative(a in arb_bpoly(), b in arb_bpoly()) {
            assert_eq!(a * b, b * a)
        }
    }

    proptest! {
        #[test]
        fn test_mul_one_identity(a in arb_bpoly()) {
            assert_eq!(a * BPoly::one(), (BPoly::zero(), a))
        }
    }

    #[test]
    fn test_one_plus_one_is_zero() {
        let one = BPoly::<4>::one();
        assert_eq!(one + one, BPoly::zero())
    }

    #[test]
    fn test_z_times_z_is_z_squared() {
        let z = BPoly::<4> { data: [2, 0, 0, 0] };
        let z2 = BPoly::<4> { data: [4, 0, 0, 0] };

        assert_eq!(z * z, (BPoly::zero(), z2))
    }
}
