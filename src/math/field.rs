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

impl<const N: usize> ops::Index<usize> for BPoly<N> {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize> ops::IndexMut<usize> for BPoly<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
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

    fn shift(&mut self, start: u64, count: usize) -> u64 {
        let mut top = start;
        for x in &mut self.data {
            let new_top = *x >> (64 - count);
            *x = (*x << count) | top;
            top = new_top;
        }
        top
    }
}

impl<const N: usize> ops::AddAssign for BPoly<N> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self[i] ^= rhs[i]
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
                if ((self[j] >> k) & 1) != 1 {
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
                let top = out_lo.shift(0, 1);
                out_hi.shift(top, 1);
            }
        }
        (out_hi, out_lo)
    }
}

#[derive(Clone, Copy, Debug)]
// Only implement equality for tests. This is to avoid the temptation to introduce
// a timing leak through equality comparison.
#[cfg_attr(test, derive(PartialEq))]
pub struct GF128(BPoly<2>);

impl GF128 {
    pub fn one() -> Self {
        Self(BPoly::one())
    }

    pub fn zero() -> Self {
        Self(BPoly::zero())
    }

    fn reduce((hi, mut lo): (BPoly<2>, BPoly<2>)) -> Self {
        // The irreducible polynomial is z^128 + z^7 + z^2 + z + 1
        for i in 0..2 {
            lo[i] ^= (hi[i] << 7) ^ (hi[i] << 2) ^ (hi[i] << 1) ^ hi[i];
            if i > 0 {
                lo[i] ^=
                    (hi[i - 1] >> (64 - 7)) ^ (hi[i - 1] >> (64 - 2)) ^ (hi[i - 1] >> (64 - 1));
            }
        }
        // The top value has at most 7 set bits, so we can safely include it as usual
        let top = (hi[1] >> (64 - 7)) ^ (hi[1] >> (64 - 2)) ^ (hi[1] >> (64 - 1));
        lo[0] ^= (top << 7) ^ (top << 2) ^ (top << 1) ^ top;
        GF128(lo)
    }
}

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

impl ops::Mul for GF128 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::reduce(self.0 * rhs.0)
    }
}

impl ops::MulAssign for GF128 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
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

    // We can generate an arbitrary element just by choosing random bits
    prop_compose! {
        fn arb_gf128()(data in any::<[u64;2]>()) -> GF128 {
            GF128(BPoly { data })
        }
    }

    proptest! {
        #[test]
        fn test_bpoly_addition_commutative(a in arb_bpoly(), b in arb_bpoly()) {
            assert_eq!(a + b, b + a)
        }
    }

    proptest! {
        #[test]
        fn test_bpoly_addition_associative(a in arb_bpoly(), b in arb_bpoly(), c in arb_bpoly()) {
            assert_eq!(a + (b + c), (a + b) + c);
        }
    }

    proptest! {
        #[test]
        fn test_bpoly_add_zero_identity(a in arb_bpoly()) {
            let zero = BPoly::zero();
            assert_eq!(a + zero, a);
            assert_eq!(zero + a, a);
        }
    }

    proptest! {
        #[test]
        fn test_bpoly_multiplication_commutative(a in arb_bpoly(), b in arb_bpoly()) {
            assert_eq!(a * b, b * a)
        }
    }

    proptest! {
        #[test]
        fn test_bpoly_mul_one_identity(a in arb_bpoly()) {
            assert_eq!(a * BPoly::one(), (BPoly::zero(), a))
        }
    }

    proptest! {
        #[test]
        fn test_gf128_multiplication_commutative(a in arb_gf128(), b in arb_gf128()) {
            assert_eq!(a * b, b * a);
        }
    }

    proptest! {
        #[test]
        fn test_gf128_multiplication_associative(a in arb_gf128(), b in arb_gf128(), c in arb_gf128()) {
            assert_eq!(a * (b * c), (a * b) * c);
        }
    }

    proptest! {
        #[test]
        fn test_gf128_mul_one_identity(a in arb_gf128()) {
            assert_eq!(a * GF128::one(), a);
        }
    }

    #[test]
    fn test_bpoly_one_plus_one_is_zero() {
        let one = BPoly::<4>::one();
        assert_eq!(one + one, BPoly::zero())
    }

    #[test]
    fn test_bpoly_z_times_z_is_z_squared() {
        let z = BPoly::<4> { data: [2, 0, 0, 0] };
        let z2 = BPoly::<4> { data: [4, 0, 0, 0] };

        assert_eq!(z * z, (BPoly::zero(), z2))
    }

    #[test]
    fn test_gf128_z127_times_z() {
        let z127 = GF128(BPoly { data: [0, 1 << 63] });
        let z = GF128(BPoly { data: [2, 0] });
        let expected = GF128(BPoly {
            data: [1 | (1 << 1) | (1 << 2) | (1 << 7), 0],
        });
        assert_eq!(z * z127, expected);
    }
}
