mod cache;
mod github;

use std::sync::{Arc, RwLock};
use tiny_http::{Server, Response, Header, StatusCode};

const HELP_RESPONSE: &str = "fbox826/last-commit-rs\n\nUsage: /{owner}/{repo}[:{branch}]?[refresh=true]\nExample: /rust-lang/rust?refresh=true";

struct AppState {
  cache: RwLock<cache::Cache>,
  agent: ureq::Agent,
  token: String,
}

fn handle(state: &AppState, req: tiny_http::Request) {
  let url = req.url().to_string();
  let (path, query) = url.split_once('?').unwrap_or((&url, ""));

  match path {
    "/" => { req.respond(Response::from_string(HELP_RESPONSE)).ok(); }
    "/health" | "/favicon.ico" => { req.respond(Response::new_empty(StatusCode(204))).ok(); }
    _ => {
      let name = &path[1..];
      let refresh = query.contains("refresh=true");

      if !refresh {
        let cache = state.cache.read().unwrap();
        if let Some(entry) = cache.get(name) {
          if !cache::is_expired(entry) {
            let header = Header::from_bytes("X-Cache", "HIT").unwrap();
            let body = entry.lastmod.clone();
            req.respond(Response::from_string(body).with_header(header)).ok();
            return;
          }
        }
      }

      let date = github::fetch(&state.agent, &state.token, name);

      if let Some(ref d) = date {
        let mut cache = state.cache.write().unwrap();
        cache.insert(name.to_string(), cache::new_entry(d.clone()));
        cache::save(&cache);
      }

      let header = Header::from_bytes("X-Cache", "MISS").unwrap();
      let date_str = date.unwrap_or_else(|| "null".to_string());
      req.respond(Response::from_string(date_str).with_header(header)).ok();
    }
  }
}

fn main() {
  let state = Arc::new(AppState {
    cache: RwLock::new(cache::load()),
    agent: github::make_agent(),
    token: format!("Bearer {}", std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set")),
  });

  let server = Server::http("[::]:3000").expect("failed to bind :3000");
  println!(":3000");

  for req in server.incoming_requests() {
    let st = Arc::clone(&state);
    std::thread::spawn(move || handle(&st, req));
  }
}
