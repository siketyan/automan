use std::convert::Infallible;

use serde::Deserialize;

use crate::events::IssueCommented;
use crate::reviewers::{Answer, Reviewer};

#[derive(Deserialize)]
pub struct Trigger {
    content: String,
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
    type Error = Infallible;

    fn review(&self, event: &IssueCommented) -> Result<Answer, Self::Error> {
        Ok(if event.content == self.trigger.content {
            Answer::Accept
        } else {
            Answer::Noop
        })
    }
}
