use octocrab::models::events::payload::{EventPayload, IssueCommentEventPayload};

pub struct IssueCommented {
    pub content: String,
}

impl From<IssueCommentEventPayload> for IssueCommented {
    fn from(payload: IssueCommentEventPayload) -> Self {
        Self {
            content: payload.comment.body.unwrap_or_default(),
        }
    }
}

pub enum Event {
    IssueCommented(IssueCommented),
}

impl TryFrom<&EventPayload> for Event {
    type Error = anyhow::Error;

    fn try_from(payload: &EventPayload) -> Result<Self, Self::Error> {
        use EventPayload::*;

        Ok(match payload {
            IssueCommentEvent(p) => Self::IssueCommented((*p.clone()).into()),
            _ => return Err(anyhow::Error::msg("Unknown payload found")),
        })
    }
}
