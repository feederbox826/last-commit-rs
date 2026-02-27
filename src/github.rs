use serde::Deserialize;
use ureq::Agent;

pub fn make_agent() -> Agent {
  Agent::config_builder()
    .https_only(true)
    .user_agent("feederbox826/last-commit/v2")
    .accept("application/vnd.github.v3+json")
    .build()
    .new_agent()
}

// API version 2022-11-28
fn api_get_json<T: serde::de::DeserializeOwned>(agent: &Agent, token: &str, url: &str) -> Option<T> {
  let mut resp = agent
    .get(url)
    .header("Authorization", token)
    .call()
    .ok()?;
  let s = resp.body_mut().read_to_string().ok()?;
  serde_json::from_str(&s).ok()
}

#[derive(Deserialize)]
struct Gist { updated_at: String }

#[derive(Deserialize)]
struct CommitAuthor { date: String }
#[derive(Deserialize)]
struct CommitInner { author: CommitAuthor }
#[derive(Deserialize)]
struct Commit { commit: CommitInner }

pub fn fetch(agent: &Agent, token: &str, name: &str) -> Option<String> {
  let is_gist = name.len() == 32 && name.bytes().all(|b| b.is_ascii_hexdigit());
  let mut date = if is_gist {
    let url = format!("https://api.github.com/gists/{}", name);
    let g: Gist = api_get_json(agent, token, &url)?;
    g.updated_at
  } else if name.contains('/') {
    let (repo_name, branch) = name.split_once(':').unwrap_or((name, "HEAD"));
    let url = format!("https://api.github.com/repos/{}/commits/{}", repo_name, branch);
    let c: Commit = api_get_json(agent, token, &url)?;
    c.commit.author.date
  } else {
    return None;
  };

  if let Some(idx) = date.find('T') { date.truncate(idx); }
  Some(date)
}
