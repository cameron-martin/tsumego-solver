use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::iter::Sum;
use std::ops::Add;

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct ProofNumber(u32);

impl ProofNumber {
    pub fn infinite() -> ProofNumber {
        ProofNumber(0)
    }

    pub fn finite(n: u32) -> ProofNumber {
        ProofNumber(n + 1)
    }
}

impl PartialOrd for ProofNumber {
    fn partial_cmp(&self, other: &ProofNumber) -> Option<Ordering> {
        match (self.0, other.0) {
            (0, 0) => Some(Ordering::Equal),
            (0, _) => Some(Ordering::Greater),
            (_, 0) => Some(Ordering::Less),
            (n, m) => n.partial_cmp(&m),
        }
    }
}

impl Ord for ProofNumber {
    fn cmp(&self, other: &ProofNumber) -> Ordering {
        match (self.0, other.0) {
            (0, 0) => Ordering::Equal,
            (0, _) => Ordering::Greater,
            (_, 0) => Ordering::Less,
            (n, m) => n.cmp(&m),
        }
    }
}

impl Sum for ProofNumber {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = 1;

        for n in iter {
            match n.0 {
                0 => return ProofNumber(0),
                m => sum += m - 1,
            }
        }

        ProofNumber(sum)
    }
}

impl Debug for ProofNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.0 {
            0 => f.write_str("∞"),
            n => Debug::fmt(&(n - 1), f),
        }
    }
}

impl Add for ProofNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self.0, rhs.0) {
            (0, _) => ProofNumber(0),
            (_, 0) => ProofNumber(0),
            (n, m) => ProofNumber((n - 1) + m),
        }
    }
}

#[cfg(tests)]
mod tests {
    #[test]
    fn infinite_is_greater_than_finite() {
        assert!(ProofNumber::infinite() > ProofNumber::finite(1));
    }
}
