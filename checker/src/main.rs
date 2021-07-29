use std::fs;
use regex::Regex;
use std::collections::HashSet;
use std::env;
use reqwest::StatusCode;
use reqwest::Error;
use serde::Deserialize;
use url::{Url, ParseError};


#[derive(Deserialize, Debug)]
struct Repo {
    updated_at: String,
}

fn is_ok(status: reqwest::StatusCode, url: &str) -> bool {
    return status == 200 || status == 403 || status == 406 ||
        (status == 502 && url.contains("reddit.com"));
}

fn path(u: &str) -> Result<Url, ParseError> {
    let parsed = Url::parse(u)?;
    return Ok(parsed);
}

fn is_repo(u: &str) -> bool {
    if !u.contains("github.com") { return false; }
    match path(u) {
        Ok(p) => return p.path().split("/").count() > 2,
        _ => return false
    }
}

async fn check_repo(u: &str) -> Result<StatusCode, Error> {
    let mut owner: &str = "";
    let mut repo: &str = "";
    if let Ok(p) = path(u) {
        let parts: Vec<&str> = p.path().split("/").collect();
        //        owner = parts[1];
        //        repo = parts[2];
        //        println!("owner/repo: {} {}", _owner, _repo);
    }

    let request_url = format!("https://api.github.com/repos/{owner}/{repo}", owner = owner, repo = repo);
    let client = reqwest::Client::builder().build()?;
    let response = reqwest::get(&request_url).await?;
    let status = response.status();
    let data: Repo = response.json().await?;
    println!("updated {} {}", status, data.updated_at);
    if !is_ok(status, u) { return Ok(status); }
    return Ok(status);
}

async fn check_url(url: &str) -> Result<StatusCode, Error> {
    let client = reqwest::Client::builder().build()?;
    let mut response = client.head(url).send().await?;
    if is_ok(response.status(), url) {
        return Ok(response.status());
    }
    response = client.get(url).send().await?;
    return Ok(response.status());
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let readme = if args.len() > 1 { &args[1] } else { "../README.md" };
    let _token = if args.len() > 2 { &args[2] } else { "" };
    let match_pattern = r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.([a-zA-Z0-9()]){1,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)";
    let contents = fs::read_to_string(readme).expect("Something went wrong reading the file");
    let re = Regex::new(match_pattern).unwrap();
    let matches: Vec<&str> = re.find_iter(&contents).filter_map(|cap| { Some(cap.as_str()) }).collect();
    let unique_urls: HashSet<_> = matches.iter().cloned().collect();
    let urls: Vec<_> = unique_urls.into_iter().collect();
    let match_count = urls.len();
    let mut i = 0;
    let mut progress = 0;
    let mut fails: Vec<&str> = Vec::new();

    println!("Checking {} entries...", match_count);
    for u in urls {
        let status_check =
            if is_repo(u) { check_repo(u).await } else { check_url(&u).await };

        match status_check {
            Ok(code) => { if !is_ok(code, u) { fails.push(&u); } }
            Err(_) => { fails.push(&u); }
        };

        i = i + 1;
        let percent = (i * 100) / match_count;
        if percent % 10 == 0 && percent > progress {
            println!("...{ } % ({})", percent, fails.len());
            progress = percent;
        }
    }

    if fails.len() > 0 {
        println!("{} failure(s):", fails.len());
        for fail in fails {
            println!("- {}", fail);
        }
        std::process::exit(1);
    }
    println!("no issues");
}
