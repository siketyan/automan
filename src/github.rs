use std::env::var;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use octocrab::models::events::payload::EventPayload;
use octocrab::Octocrab;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(Error::custom)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Internal,
    Private,
}

#[derive(Deserialize)]
pub struct Context {
    pub token: String,
    pub job: String,
    #[serde(rename = "ref")]
    pub ref_: String,
    pub sha: String,
    pub repository: String,
    pub repository_owner: String,
    #[serde(rename = "repositoryUrl")]
    pub repository_url: Url,
    #[serde(deserialize_with = "from_str")]
    pub run_id: usize,
    #[serde(deserialize_with = "from_str")]
    pub run_number: usize,
    #[serde(deserialize_with = "from_str")]
    pub retention_days: usize,
    #[serde(deserialize_with = "from_str")]
    pub run_attempt: usize,
    #[serde(deserialize_with = "from_str")]
    pub artifact_cache_size_limit: usize,
    pub repository_visibility: Visibility,
    pub repository_id: String,
    pub actor_id: String,
    pub actor: String,
    pub workflow: String,
    pub head_ref: String,
    pub base_ref: String,
    pub event: EventPayload,
}

impl Context {
    pub fn from_env() -> Result<Self> {
        Ok(serde_json::from_str(&(var("INPUT_CONTEXT")?))?)
    }
}

pub struct PullRequestReviewHandler<'octo> {
    crab: &'octo Octocrab,
    owner: String,
    repo: String,
}

impl<'octo> PullRequestReviewHandler<'octo> {
    pub(crate) fn new(
        crab: &'octo Octocrab,
        owner: impl Into<String>,
        repo: impl Into<String>,
    ) -> Self {
        Self {
            crab,
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn create<'b>(
        &'b self,
        pull_number: u64,
        event: PullRequestReviewEvent,
    ) -> CreatePullRequestReviewHandler<'octo, 'b> {
        CreatePullRequestReviewHandler::new(self, pull_number, event)
    }

    pub(crate) async fn http_post<R, A, P>(&self, route: A, body: Option<&P>) -> octocrab::Result<R>
    where
        A: AsRef<str>,
        P: serde::Serialize + ?Sized,
        R: octocrab::FromResponse,
    {
        self.crab.post(route, body).await
    }
}

#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PullRequestReviewEvent {
    Approve,
    Comment,
    RequestChanges,
}

#[derive(Serialize)]
pub struct CreatePullRequestReviewHandler<'octo, 'b> {
    #[serde(skip)]
    handler: &'b PullRequestReviewHandler<'octo>,
    #[serde(skip)]
    pull_number: u64,
    event: PullRequestReviewEvent,
    body: Option<String>,
    commit_id: Option<String>,
}

impl<'octo, 'b> CreatePullRequestReviewHandler<'octo, 'b> {
    pub fn new(
        handler: &'b PullRequestReviewHandler<'octo>,
        pull_number: u64,
        event: PullRequestReviewEvent,
    ) -> Self {
        Self {
            handler,
            pull_number,
            event,
            body: None,
            commit_id: None,
        }
    }

    pub fn body<A: Into<String>>(mut self, body: impl Into<Option<A>>) -> Self {
        self.body = body.into().map(|b| b.into());
        self
    }

    pub fn commit_id<A: Into<String>>(mut self, commit_id: impl Into<Option<A>>) -> Self {
        self.commit_id = commit_id.into().map(|c| c.into());
        self
    }

    pub async fn send(self) -> octocrab::Result<octocrab::models::pulls::Review> {
        let url = format!(
            "repos/{owner}/{repo}/pulls/{pr}/reviews",
            owner = self.handler.owner,
            repo = self.handler.repo,
            pr = self.pull_number,
        );
        self.handler.http_post(url, Some(&self)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use octocrab::models::events::payload::PullRequestEventAction;

    #[test]
    fn deserialize_pull_request_synchronize() {
        let json = include_str!("../resources/pull-request-synchronize.json");
        let context: Context = serde_json::from_str(json).unwrap();

        assert_eq!("test", context.job);
        assert_matches!(context.event, EventPayload::PullRequestEvent(e) => {
            assert_matches!(e.action, PullRequestEventAction::Synchronize);
        });
    }

    #[test]
    fn deserialize_pull_request_edited() {
        let json = include_str!("../resources/pull-request-edited.json");
        let context: Context = serde_json::from_str(json).unwrap();

        assert_matches!(context.event, EventPayload::PullRequestEvent(e) => {
            assert_matches!(e.action, PullRequestEventAction::Edited);
        });
    }
}
