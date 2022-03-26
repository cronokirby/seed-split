use super::field;
use rand_core::{CryptoRng, RngCore};

struct Polynomial<F> {
    coefficients: Vec<F>,
}

impl<F: field::Field> Polynomial<F> {
    fn random<R: RngCore + CryptoRng>(rng: &mut R, zero_value: F, degree: usize) -> Self {
        let mut coefficients = Vec::with_capacity(degree + 1);
        coefficients.push(zero_value);
        for _ in 0..degree {
            coefficients.push(F::random(rng));
        }
        Polynomial { coefficients }
    }

    fn evaluate(&self, at: F) -> F {
        let mut acc = *self.coefficients.last().unwrap();
        for &c in self.coefficients.iter().rev().skip(1) {
            acc = acc * at + c;
        }
        acc
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Index(u8);

impl Index {
    fn to_field<F: field::Field>(self) -> F {
        F::from(u64::from(self.0) + 1)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sharing {
    threshold: u8,
    count: u8,
}

impl Sharing {
    pub fn new(threshold: u8, count: u8) -> Self {
        debug_assert!(threshold > 0 && threshold <= count);
        Self { threshold, count }
    }

    pub fn validate_index(&self, index: u8) -> Option<Index> {
        if index < self.count {
            Some(Index(index))
        } else {
            None
        }
    }
}

pub fn share<F: field::Field, R: RngCore + CryptoRng>(
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
