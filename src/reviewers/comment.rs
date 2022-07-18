use anyhow::Error;
use regex::Regex;
use serde::Deserialize;

use crate::events::IssueCommented;
use crate::reviewers::{Answer, Reviewer};

#[derive(Deserialize)]
pub struct Trigger {
    content: String,
    #[serde(default)]
    regex: bool,
    #[serde(default)]
    inverse: bool,
}

pub struct CommentReviewer {
    trigger: Trigger,
}

impl From<Trigger> for CommentReviewer {
    fn from(trigger: Trigger) -> Self {
        Self { trigger }
    }
}

impl Reviewer<IssueCommented> for CommentReviewer {
    type Error = Error;

    fn review(&self, event: &IssueCommented) -> Result<Answer, Self::Error> {
        let mut ok = match self.trigger.regex {
            true => Regex::new(&self.trigger.content)?.is_match(&event.content),
            _ => event.content == self.trigger.content,
        };

        if self.trigger.inverse {
            ok = !ok;
        }

        Ok(match ok {
            true => Answer::Accept,
            _ => Answer::Noop,
        })
    }
}
