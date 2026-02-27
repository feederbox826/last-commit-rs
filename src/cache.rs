use std::collections::HashMap;
use std::fs::{write, read_to_string};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::Write;

const WEEK: u64 = 7 * 24 * 3600;
const CACHE_FILE: &str = "repo_cache.tsv";

pub struct CacheEntry {
  pub lastmod: String,
  pub exp: u64,
}

pub type Cache = HashMap<String, CacheEntry>;

pub fn now() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub fn is_expired(entry: &CacheEntry) -> bool {
  now() > entry.exp
}

pub fn new_entry(date: String) -> CacheEntry {
  CacheEntry {
    lastmod: date,
    exp: now() + WEEK,
  }
}

pub fn load() -> Cache {
  let now = now();
  read_to_string(CACHE_FILE)
    .unwrap_or_default()
    .lines()
    .filter_map(|line| {
      let mut parts = line.splitn(3, '\t');
      let key = parts.next()?;
      let lastmod = parts.next()?;
      let exp = parts.next()?.parse::<u64>().ok()?;
      if exp > now {
        Some((key.to_owned(), CacheEntry { lastmod: lastmod.to_owned(), exp }))
      } else {
        None
      }
    })
    .collect()
}

pub fn save(cache: &Cache) {
  let mut out = String::with_capacity(cache.len().saturating_mul(32));
  for (k, v) in cache {
    let _ = write!(&mut out, "{}\t{}\t{}\n", k, v.lastmod, v.exp);
  }
  let _ = write(CACHE_FILE, out);
}
