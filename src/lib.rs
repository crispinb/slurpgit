use std::{error::Error, fmt::Display};

use octocrab::{models::Repository, Octocrab};
use serde::Serialize;
use url::Url;

// TODO: obfuscate private repo names & urls? Or require auth?
// TODO: write very simple & literal list of fly.io deployment steps

type SlurpError = Box<dyn Error + Send + Sync + 'static>;
type Result<T> = std::result::Result<T, SlurpError>;

#[derive(Debug, Serialize)]
pub enum RepoType {
    Source,
    Fork,
}

#[derive(Debug, Serialize)]
pub struct UserRepo {
    id: u64,
    url: Url,
    name: String,
    repo_type: RepoType,
    description: Option<String>,
    private: bool,
    archived: bool,
}

// NB: Octo is an opaque type, hard to test.
//     Could do a serde deser from a previously serialised, but seems overwrought
// NB: Unwrapping some of the Option values for research - no idea why they would be
impl From<Repository> for UserRepo {
    fn from(value: Repository) -> Self {
        use RepoType::*;

        let id = value.id.0;
        let url = value.html_url.unwrap();
        let name = value.name;
        let description = value.description;
        let private = value.private.unwrap();
        let repo_type = if value.fork.unwrap() { Fork } else { Source };
        let archived = value.private.unwrap();

        Self {
            id,
            url,
            name,
            repo_type,
            description,
            private,
            archived,
        }
    }
}

impl Display for UserRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (private? {}), url: {}",
            self.name, self.private, self.url
        )
    }
}

pub async fn repositories() -> Result<Vec<UserRepo>> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var required");
    let client = Octocrab::builder().personal_token(token).build()?;
    let repos = client
        .current()
        .list_repos_for_authenticated_user()
        .type_("owner")
        .sort("updated")
        .send()
        .await?;

    let mut user_repos = vec![];
    for repo in repos {
        let repo: UserRepo = repo.into();
        user_repos.push(repo);
    }

    Ok(user_repos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_issues() {
        let repos = repositories().await.unwrap();
        println!("Found {} repos", repos.len());
        for repo in repos {
            println!("{}", repo);
        }
    }

    #[tokio::test]
    async fn get_issues_json() {
        let repos = repositories().await.unwrap();
        println!("Found {} repos\nJSON reps: ", repos.len());
        for repo in repos {
            println!("{}", serde_json::to_string(&repo).unwrap());
        }
    }

    #[tokio::test]
    async fn option_rendering() {
        let repo = UserRepo {
            id: 1,
            name: "Fark".into(),
            url: Url::parse("http://github.com/crispinb").unwrap(),
            repo_type: RepoType::Source,
            description: None,
            private: true,
            archived: false,
        };
        println!(
            "rep with a missing descr: \n{}",
            serde_json::to_string(&repo).unwrap()
        );
    }
}
