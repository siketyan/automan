pub mod events;
pub mod github;
pub mod reviewers;

use std::fs::read_to_string;
use std::iter::Iterator;
use std::path::Path;

use anyhow::Result;
use octocrab::models::events::payload::EventPayload;
use octocrab::OctocrabBuilder;
use serde::Deserialize;

use crate::events::Event;
use crate::github::{Context, PullRequestReviewEvent, PullRequestReviewHandler};
use crate::reviewers::comment::CommentReviewer;
use crate::reviewers::description::DescriptionReviewer;
use crate::reviewers::{Answer, Reviewer};

#[derive(Deserialize)]
pub struct Triggers {
    pub comment: Option<Vec<reviewers::comment::Trigger>>,
    pub description: Option<Vec<reviewers::description::Trigger>>,
}

#[derive(Deserialize)]
pub struct Config {
    pub triggers: Triggers,
    pub comment: String,
    pub default: Answer,
}

impl Config {
    pub fn from_yaml(path: impl AsRef<Path>) -> Result<Self> {
        Ok(serde_yaml::from_str(&read_to_string(path)?)?)
    }
}

#[derive(Default)]
pub struct EventReviewers {
    pub comment: Vec<CommentReviewer>,
    pub description: Vec<DescriptionReviewer>,
}

impl From<Triggers> for EventReviewers {
    fn from(triggers: Triggers) -> Self {
        Self {
            comment: triggers
                .comment
                .unwrap_or_default()
                .into_iter()
                .map(|t| t.into())
                .collect(),
            description: triggers
                .description
                .unwrap_or_default()
                .into_iter()
                .map(|t| t.into())
                .collect(),
        }
    }
}

fn fold<E, R>(e: E) -> impl Fn(Answer, R) -> std::result::Result<Answer, <R as Reviewer<E>>::Error>
where
    R: Reviewer<E>,
{
    move |acc, r| r.review(&e).map(|a| Answer::choose(acc, a))
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_yaml("./.automan.yaml")?;
    let reviewers = EventReviewers::from(config.triggers);
    let context = Context::from_env()?;
    let default = config.default;
    let answer = match Event::try_from(&context.event)? {
        Event::IssueCommented(e) => reviewers.comment.into_iter().try_fold(default, fold(e)),
        Event::DescriptionEdited(e) => reviewers.description.into_iter().try_fold(default, fold(e)),
    }?;

    let event = match answer {
        Answer::Accept | Answer::ForceAccept => PullRequestReviewEvent::Approve,
        Answer::Reject => PullRequestReviewEvent::RequestChanges,
        _ => return Ok(()),
    };

    let repo = context.repository.split("/").last().unwrap_or_default();
    let pull_number = match &context.event {
        EventPayload::IssueCommentEvent(p) => Some(p.issue.number as u64),
        EventPayload::PullRequestEvent(p) => Some(p.number),
        _ => None,
    };

    let octocrab = OctocrabBuilder::new()
        .personal_token(context.token)
        .build()?;

    PullRequestReviewHandler::new(&octocrab, context.repository_owner, repo)
        .create(pull_number.unwrap_or_default(), event)
        .body(config.comment)
        .commit_id(context.sha)
        .send()
        .await?;

    Ok(())
}
