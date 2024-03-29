use anyhow::Error;
use serde::Deserialize;

use crate::events::IssueCommented;
use crate::reviewers::{Answer, Reviewer, TextMatch};

#[derive(Deserialize)]
pub struct Trigger {
    #[serde(flatten)]
    pub text: TextMatch,
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
        Ok(match self.trigger.text.matches(&event.content)? {
            true => Answer::Accept,
            _ => Answer::Noop,
        })
    }
}
