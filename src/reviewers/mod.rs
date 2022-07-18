pub mod comment;
pub mod description;

use core::result::Result;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Answer {
    Noop = 0,
    #[serde(rename = "weak")]
    WeakReject = 10,
    #[serde(skip)]
    Reject = 20,
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

#[derive(Debug, Clone, Deserialize)]
pub struct TextMatch {
    content: String,
    #[serde(default)]
    regex: bool,
    #[serde(default)]
    inverse: bool,
}

impl TextMatch {
    pub fn matches(&self, content: &str) -> anyhow::Result<bool> {
        let ok = match self.regex {
            true => Regex::new(&self.content)?.is_match(content),
            _ => content == self.content,
        };

        Ok(match self.inverse {
            true => !ok,
            _ => ok,
        })
    }
}

pub trait Reviewer<E> {
    type Error;

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
        assert_eq!(Accept, Answer::choose(WeakReject, Accept));
    }
}
