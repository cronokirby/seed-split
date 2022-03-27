use rand_core::RngCore;
use std::ops;
use subtle::{Choice, ConditionallySelectable};

/// Represents some kind of field.
///
/// We require addition and multiplication, along with inversion.
///
/// We require copy mainly for convenience.
pub trait Field:
    Copy
    + ops::Add<Output = Self>
    + ops::AddAssign
    + ops::Sub<Output = Self>
    + ops::SubAssign
    + ops::Neg<Output = Self>
    + ops::Mul<Output = Self>
    + ops::MulAssign
    + From<u64>
{
    /// Return the multiplicative inverse of this element.
    fn inverse(self) -> Self;
    /// Return the multlicative unit in this field.
    fn one() -> Self;
    /// Return the additive identity in the field.
    fn zero() -> Self;
    /// Create a random element of this field.
    fn random<R: RngCore>(rng: &mut R) -> Self;
}

// This function is useful to do inversion in a field of size 2^count.
fn exp_two_count_minus_two<M: Copy + ops::MulAssign>(count: usize, mut acc: M, x: M) -> M {
    for _ in 0..(count - 1) {
        acc *= acc;
        acc *= x;
    }
    acc *= acc;
    acc
}

/// Represents a binary polynomial with 64 * N coefficients.
/// 
/// This is useful as an intermediate building block towards building binary
/// fields, which use polynomials for their arithmetic.
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

impl<const N: usize> ConditionallySelectable for BPoly<N> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut out = BPoly::zero();
        for i in 0..N {
            out[i] = u64::conditional_select(&a[i], &b[i], choice);
        }
        out
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

    /// Shift this polynomial to the left by count.
    /// 
    /// We also take a starting value, which will be put into the empty low bits
    /// of the first limb.
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
                let to_add = Self::conditional_select(
                    &Self::zero(),
                    &rhs,
                    Choice::from(((self[j] >> k) & 1) as u8),
                );
                // Hopefully all of this can get inlined
                let mut view = Self::zero();
                view.data[..(N - j)].copy_from_slice(&out_lo.data[j..]);
                view.data[(N - j)..].copy_from_slice(&out_hi.data[..j]);
                view += to_add;
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

/// Represents the binary field GF(2^128).
#[derive(Clone, Copy, Debug)]
// Only implement equality for tests. This is to avoid the temptation to introduce
// a timing leak through equality comparison.
#[cfg_attr(test, derive(PartialEq))]
pub struct GF128(BPoly<2>);

impl GF128 {
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

impl From<u64> for GF128 {
    fn from(x: u64) -> Self {
        Self(BPoly { data: [x, 0] })
    }
}

impl Field for GF128 {
    fn inverse(self) -> Self {
        exp_two_count_minus_two(128, Self::one(), self)
    }

    fn one() -> Self {
        Self(BPoly::one())
    }

    fn zero() -> Self {
        Self(BPoly::zero())
    }

    fn random<R: RngCore>(rng: &mut R) -> Self {
        let mut buf = [0; 16];
        rng.fill_bytes(&mut buf);
        Self::from(buf)
    }
}

impl Into<[u8; 16]> for GF128 {
    fn into(self) -> [u8; 16] {
        let mut out = [0; 16];
        for i in 0..2 {
            out[8 * i..8 * (i + 1)].copy_from_slice(&self.0[i].to_le_bytes())
        }
        out
    }
}

impl From<[u8; 16]> for GF128 {
    fn from(data: [u8; 16]) -> Self {
        let mut out = Self::zero();
        for (i, chunk) in data.chunks_exact(8).enumerate() {
            out.0[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        }
        out
    }
}

/// Represents the binary field GF(2^256).
#[derive(Clone, Copy, Debug)]
// Only implement equality for tests. This is to avoid the temptation to introduce
// a timing leak through equality comparison.
#[cfg_attr(test, derive(PartialEq))]
pub struct GF256(BPoly<4>);

impl GF256 {
    fn reduce((hi, mut lo): (BPoly<4>, BPoly<4>)) -> Self {
        // The irreducible polynomial is z^256 + z^10 + z^5 + z^2 + 1
        for i in 0..4 {
            lo[i] ^= (hi[i] << 10) ^ (hi[i] << 5) ^ (hi[i] << 2) ^ hi[i];
            if i > 0 {
                lo[i] ^=
                    (hi[i - 1] >> (64 - 10)) ^ (hi[i - 1] >> (64 - 5)) ^ (hi[i - 1] >> (64 - 2));
            }
        }
        // The top value has at most 10 set bits, so we can safely include it as usual
        let top = (hi[3] >> (64 - 10)) ^ (hi[3] >> (64 - 5)) ^ (hi[3] >> (64 - 2));
        lo[0] ^= (top << 10) ^ (top << 5) ^ (top << 2) ^ top;
        GF256(lo)
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

impl ops::AddAssign for GF256 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl ops::Neg for GF256 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self
    }
}

impl ops::Sub for GF256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl ops::SubAssign for GF256 {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}

impl ops::Mul for GF256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::reduce(self.0 * rhs.0)
    }
}

impl ops::MulAssign for GF256 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl From<u64> for GF256 {
    fn from(x: u64) -> Self {
        Self(BPoly { data: [x, 0, 0, 0] })
    }
}

impl Field for GF256 {
    fn inverse(self) -> Self {
        exp_two_count_minus_two(256, Self::one(), self)
    }

    fn one() -> Self {
        Self(BPoly::one())
    }

    fn zero() -> Self {
        Self(BPoly::zero())
    }

    fn random<R: RngCore>(rng: &mut R) -> Self {
        let mut buf = [0; 32];
        rng.fill_bytes(&mut buf);
        Self::from(buf)
    }
}

impl Into<[u8; 32]> for GF256 {
    fn into(self) -> [u8; 32] {
        let mut out = [0; 32];
        for i in 0..4 {
            out[8 * i..8 * (i + 1)].copy_from_slice(&self.0[i].to_le_bytes())
        }
        out
    }
}

impl From<[u8; 32]> for GF256 {
    fn from(data: [u8; 32]) -> Self {
        let mut out = Self::zero();
        for (i, chunk) in data.chunks_exact(8).enumerate() {
            out.0[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        }
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

    // We can generate an arbitrary element just by choosing random bits
    prop_compose! {
        fn arb_gf128()(data in any::<[u64;2]>()) -> GF128 {
            GF128(BPoly { data })
        }
    }

    // We can generate an arbitrary element just by choosing random bits
    prop_compose! {
        fn arb_gf256()(data in any::<[u64;4]>()) -> GF256 {
            GF256(BPoly { data })
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

    proptest! {
        #[test]
        fn test_gf128_mul_inverse_is_one(a in arb_gf128()) {
            if a != GF128::zero() {
                assert_eq!(a * a.inverse(), GF128::one());
            }
        }
    }

    proptest! {
        #[test]
        fn test_gf256_multiplication_commutative(a in arb_gf256(), b in arb_gf256()) {
            assert_eq!(a * b, b * a);
        }
    }

    proptest! {
        #[test]
        fn test_gf256_multiplication_associative(a in arb_gf256(), b in arb_gf256(), c in arb_gf256()) {
            assert_eq!(a * (b * c), (a * b) * c);
        }
    }

    proptest! {
        #[test]
        fn test_gf256_mul_one_identity(a in arb_gf256()) {
            assert_eq!(a * GF256::one(), a);
        }
    }

    proptest! {
        #[test]
        fn test_gf256_mul_inverse_is_one(a in arb_gf256()) {
            if a != GF256::zero() {
                assert_eq!(a * a.inverse(), GF256::one());
            }
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
