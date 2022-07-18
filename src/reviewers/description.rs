use anyhow::Error;
use serde::Deserialize;

use crate::events::DescriptionEdited;
use crate::reviewers::{Answer, Reviewer, TextMatch};

#[derive(Deserialize)]
pub struct Trigger {
    #[serde(flatten)]
    pub text: TextMatch,
}

pub struct DescriptionReviewer {
    trigger: Trigger,
}

impl From<Trigger> for DescriptionReviewer {
    fn from(trigger: Trigger) -> Self {
        Self { trigger }
    }
}

impl Reviewer<DescriptionEdited> for DescriptionReviewer {
    type Error = Error;

    fn review(&self, event: &DescriptionEdited) -> Result<Answer, Self::Error> {
        Ok(match self.trigger.text.matches(&event.content)? {
            true => Answer::Accept,
            _ => Answer::Noop,
        })
    }
}
