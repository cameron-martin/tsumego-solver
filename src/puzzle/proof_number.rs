use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::iter::Sum;
use std::ops::Add;
use std::ops::Sub;

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
        Some(self.cmp(other))
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

impl Sub for ProofNumber {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        match (self.0, rhs.0) {
            (_, 0) => panic!("Cannot subtract infinity"),
            (0, _) => ProofNumber::infinite(),
            (n, m) => {
                if m <= n {
                    ProofNumber::finite(n - m)
                } else {
                    panic!("Cannot subtract when result is negative");
                }
            }
        }
    }
}

#[cfg(tests)]
mod tests {
    #[test]
    fn infinite_is_greater_than_finite() {
        assert!(ProofNumber::infinite() > ProofNumber::finite(1));
    }

    #[test]
    #[should_panic]
    fn infinity_subtract_infinity_errors() {
        ProofNumber::infinite() - ProofNumber::infinite()
    }

    #[test]
    fn infinity_subtract_finite_is_infinity() {
        assert_eq!(
            ProofNumber::infinite() - ProofNumber::finite(1),
            ProofNumber::infinite()
        );
    }

    #[test]
    #[should_panic]
    fn finite_subtract_infinity_errors() {
        ProofNumber::finite(1) - ProofNumber::infinite()
    }

    #[test]
    fn finite_subtract_finite_succeeds_if_result_is_positive() {
        assert_eq!(
            ProofNumber::finite(10) - ProofNumber::finite(4),
            ProofNumber::finite(6)
        );
    }

    #[test]
    fn finite_subtract_finite_succeeds_if_result_is_zero() {
        assert_eq!(
            ProofNumber::finite(10) - ProofNumber::finite(10),
            ProofNumber::finite(0)
        );
    }

    #[test]
    #[should_panic]
    fn finite_subtract_finite_errors_if_result_is_negative() {
        ProofNumber::finite(4) - ProofNumber::finite(10)
    }
}
