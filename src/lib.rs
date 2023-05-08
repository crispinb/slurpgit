use std::{error::Error, fmt::Display};

use octocrab::{models::Repository, Octocrab};
use serde::Serialize;
use url::Url;

// TODO: obfuscate private repo names & urls? Or require auth?

type SlurpError = Box<dyn Error + Send + Sync + 'static>;
type Result<T> = std::result::Result<T, SlurpError>;

#[derive(Debug, Serialize)]
pub enum RepoType {
    Source,
    Fork,
}

impl Display for RepoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            RepoType::Source => "Source".into(),
            RepoType::Fork => "Fork".into(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize)]
pub struct UserRepo {
    id: u64,
    pub url: Option<Url>,
    // TODO: figure out better way
    pub name: String,
    pub repo_type: RepoType,
    pub description: Option<String>,
    pub private: bool,
    archived: bool,
}

// NB: Octo is an opaque type, hard to test.
//     Could do a serde deser from a previously serialised, but seems overwrought
// NB: `unwraps` on Repository values left here for research. I'm hoping they'll
//      panic so I can find out why/when they're Options
impl From<Repository> for UserRepo {
    fn from(value: Repository) -> Self {
        use RepoType::*;

        let id = value.id.0;
        let description = value.description;
        let private = value.private.unwrap();
        let repo_type = if value.fork.unwrap() { Fork } else { Source };
        let archived = value.private.unwrap();
        let (url, name) = if private {
            (None, String::from("*****"))
        } else {
            (value.html_url, value.name)
        };

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

impl UserRepo {
    pub fn url(&self) -> String {
        match &self.url {
            Some(url) => url.to_string(),
            None => "http://240.0.0.0".into(),
        }
    }

    pub fn url_anchor(&self) -> String {
        match &self.url {
            Some(url) => format!(r#"<a href="{}">{} on Github</a>"#, url, self.name),
            None => "[private repo]".into(),
        }
    }
}


impl Display for UserRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (private? {}), url: {}",
            self.name, self.private, self.url()
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
