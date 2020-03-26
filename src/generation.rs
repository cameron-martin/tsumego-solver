mod candidate;
mod validation;

use crate::go::GoBoard;
pub use candidate::generate_candidate;
use std::time::Duration;
pub use validation::validate_candidate;

pub fn generate_puzzle(timeout: Duration) -> GoBoard {
    loop {
        let candidate = generate_candidate();

        if validate_candidate(candidate, timeout) {
            return candidate;
        }
    }
}
