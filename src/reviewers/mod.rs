pub mod comment;

use std::error::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Answer {
    Noop = 0,
    Reject = 10,
    Accept = 90,
    ForceAccept = 100,
}

impl Answer {
    pub fn choose(a: Self, b: Self) -> Self {
        use Answer::*;

        if (a == Reject || b == Reject) && (a != ForceAccept && b != ForceAccept) {
            Reject
        } else {
            Ord::max(a, b)
        }
    }
}

pub trait Reviewer<E> {
    type Error: Error;

    fn review(&self, event: &E) -> Result<Answer, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Answer::*;

    #[test]
    fn test_choose() {
        assert_eq!(Reject, Answer::choose(Reject, Accept));
        assert_eq!(Accept, Answer::choose(Accept, Noop));
        assert_eq!(Noop, Answer::choose(Noop, Noop));
        assert_eq!(ForceAccept, Answer::choose(Reject, ForceAccept));
    }
}
