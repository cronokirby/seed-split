use super::field;
use rand_core::{CryptoRng, RngCore};

/// Represents a polynomial with coefficients over some field.
struct Polynomial<F> {
    /// The coefficients, from low order to high order.
    coefficients: Vec<F>,
}

impl<F: field::Field> Polynomial<F> {
    /// Return a polynomial with random coefficients, aside from the first.
    fn random<R: RngCore + CryptoRng>(rng: &mut R, zero_value: F, degree: usize) -> Self {
        let mut coefficients = Vec::with_capacity(degree + 1);
        coefficients.push(zero_value);
        for _ in 0..degree {
            coefficients.push(F::random(rng));
        }
        Polynomial { coefficients }
    }

    /// Evaluate this polynomial, considered as a function.
    fn evaluate(&self, at: F) -> F {
        let mut acc = *self.coefficients.last().unwrap();
        for &c in self.coefficients.iter().rev().skip(1) {
            acc = acc * at + c;
        }
        acc
    }
}

/// Represents an index of a given share.
/// 
/// This is really just a wrapper over a u8.
#[derive(Clone, Copy, Debug)]
pub struct Index(u8);

impl Index {
    fn to_field<F: field::Field>(self) -> F {
        F::from(u64::from(self.0) + 1)
    }
}

impl From<u8> for Index {
    fn from(x: u8) -> Self {
        Index(x)
    }
}

impl From<Index> for u8 {
    fn from(x: Index) -> Self {
        x.0
    }
}

/// Represents a configurationg for secret sharing.
/// 
/// This holds a threshold, as well as the total number of shares.
/// This always holds a valid configuration.
#[derive(Clone, Copy, Debug)]
pub struct Sharing {
    threshold: u8,
    count: u8,
}

impl Sharing {
    /// Create a new sharing from a threshold and a count.
    /// 
    /// The threshold must be greater than 0, and <= count.
    pub fn new(threshold: u8, count: u8) -> Self {
        debug_assert!(threshold > 0 && threshold <= count);
        Self { threshold, count }
    }
}

/// Split a secret into multiple shares.
/// 
/// This function will return different shares on each invocation, using
/// the provided random generator.
pub fn split<F: field::Field, R: RngCore + CryptoRng>(
    rng: &mut R,
    secret: F,
    sharing: Sharing,
) -> Vec<(Index, F)> {
    let poly = Polynomial::random(rng, secret, usize::from(sharing.threshold - 1));
    let mut acc = Vec::with_capacity(sharing.count.into());
    for i in 0..sharing.count {
        let index = Index(i);
        let f_index = poly.evaluate(index.to_field());
        acc.push((index, f_index))
    }
    acc
}

/// A convenience struct to hold the points we've evaluated the polynomial at.
struct EvaluationPoints<F> {
    points: Vec<F>,
    heights: Vec<F>,
}

impl<F: field::Field> EvaluationPoints<F> {
    fn lagrange_coefficient(&self, j: usize) -> F {
        let mut top = F::one();
        let mut bot = F::one();
        for (i, &a_i) in self.points.iter().enumerate() {
            if i == j {
                continue;
            }
            top *= a_i;
            bot *= a_i - self.points[j];
        }
        top * bot.inverse()
    }

    // Reconstruct the zero coefficient of this polynomial
    fn reconstruct_zero(&self) -> F {
        let mut out = F::zero();
        for (j, &f_j) in self.heights.iter().enumerate() {
            out += self.lagrange_coefficient(j) * f_j;
        }
        out
    }

    fn from_shares(shares: &[(Index, F)]) -> Self {
        let points = shares.iter().map(|(i, _)| i.to_field()).collect();
        let heights = shares.iter().map(|(_, f)| *f).collect();
        Self { points, heights }
    }
}

/// Reconstruct a secret from a combination of shares.
/// 
/// If insufficient shares are provided, then this function will silently
/// return an incorrect result.
pub fn reconstruct<F: field::Field>(shares: &[(Index, F)]) -> F {
    EvaluationPoints::from_shares(shares).reconstruct_zero()
}

#[cfg(test)]
mod test {
    use crate::math::field::Field;

    use super::*;
    use field::GF128;
    use rand_core::OsRng;

    #[test]
    fn test_share_reconstruction() {
        let mut rng = &mut OsRng;
        let secret = GF128::random(&mut rng);
        let sharing = Sharing::new(5, 5);
        let shares = split(&mut rng, secret, sharing);
        let reconstructed = reconstruct(&shares);
        assert_eq!(secret, reconstructed);
    }
}
