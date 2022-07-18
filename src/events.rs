use octocrab::models::events::payload::{
    EventPayload, IssueCommentEventPayload, PullRequestEventPayload,
};

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

pub struct DescriptionEdited {
    pub content: String,
}

impl From<PullRequestEventPayload> for DescriptionEdited {
    fn from(payload: PullRequestEventPayload) -> Self {
        Self {
            content: payload.pull_request.body.unwrap_or_default(),
        }
    }
}

pub enum Event {
    IssueCommented(IssueCommented),
    DescriptionEdited(DescriptionEdited),
}

impl TryFrom<&EventPayload> for Event {
    type Error = anyhow::Error;

    fn try_from(payload: &EventPayload) -> Result<Self, Self::Error> {
        use EventPayload::*;

        Ok(match payload {
            IssueCommentEvent(p) => Self::IssueCommented((*p.clone()).into()),
            PullRequestEvent(p) => Self::DescriptionEdited((*p.clone()).into()),
            _ => return Err(anyhow::Error::msg("Unknown payload found")),
        })
    }
}
