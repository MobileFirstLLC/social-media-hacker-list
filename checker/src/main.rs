use chrono::offset::Utc;
use chrono::DateTime;
use regex::Regex;
use reqwest::{Client, Error, StatusCode};
use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use url::{Host, Url};

enum CheckResult {
    Success(),
    Error(String),
}

#[derive(Deserialize)]
struct Repo {
    updated_at: DateTime<Utc>,
}

fn is_ok(status: StatusCode, url: &str) -> bool {
    return status == 200
        || status == 403
        || status == 406
        || (status == 502 && url.contains("reddit.com/"))
        || (status == 429 && url.contains("apps.apple.com/"));
}

fn get_url(url: &str) -> Url {
    return Url::parse(url).expect("Invalid url");
}

fn is_repo(url: &str) -> bool {
    let res = get_url(url);
    return res.host() == Some(Host::Domain("github.com")) && res.path().split("/").count() > 2;
}

fn gh_repo_url(url: &str) -> String {
    let res = get_url(url);
    let parts: Vec<&str> = res.path().split("/").collect();
    return format!("https://api.github.com/repos/{}/{}", parts[1], parts[2]);
}

fn is_active(date: DateTime<Utc>) -> bool {
    return Utc::now().signed_duration_since(date).num_days() < 365;
}

async fn check_repo(u: &str, token: &str) -> Result<CheckResult, Error> {
    let response = Client::new()
        .get(&gh_repo_url(u))
        .header("User-Agent", "url-checker")
        .header("Accept", "application/vnd.github.v3+json")
        .header("Authorization", format!("token {}", token))
        .send()
        .await?;
    let status = response.status();
    if is_ok(status, u) {
        let repo: Repo = response.json().await?;
        if is_active(repo.updated_at) {
            return Ok(CheckResult::Success());
        }
        return Ok(CheckResult::Error(String::from("inactive")));
    }
    return Ok(CheckResult::Error(status.to_string()));
}

async fn check_url(u: &str) -> Result<CheckResult, Error> {
    let head = Client::new()
        .head(u)
        .header("Accept", "*/*")
        .header("User-Agent", "url-checker")
        .send()
        .await?;
    if is_ok(head.status(), u) {
        return Ok(CheckResult::Success());
    }
    let get = Client::new()
        .get(u)
        .header("Accept", "*/*")
        .header("User-Agent", "url-checker")
        .send()
        .await?;
    if is_ok(get.status(), u) {
        return Ok(CheckResult::Success());
    }
    return Ok(CheckResult::Error(get.status().to_string()));
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let readme = if args.len() > 1 {
        &args[1]
    } else {
        "../README.md"
    };
    let token = if args.len() > 2 { &args[2] } else { "" };
    let pattern = r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.([a-zA-Z0-9()]){1,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)";
    let contents = read_to_string(readme).expect("File read error!");
    let re = Regex::new(pattern).unwrap();
    let matches: Vec<&str> = re
        .find_iter(&contents)
        .filter_map(|cap| Some(cap.as_str()))
        .collect();
    let urls: Vec<_> = matches
        .iter()
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let total = urls.len();
    let mut fails: Vec<String> = Vec::new();
    let mut progress = 0;
    let mut index = 0;

    println!("Checking {} entries...", total);
    for u in urls {
        let check_result;
        if is_repo(u) {
            check_result = check_repo(u, token).await;
        } else {
            check_result = check_url(u).await;
        }
        match check_result {
            Ok(res) => match res {
                CheckResult::Error(reason) => fails.push(format!("{} - {}", u, reason)),
                _ => {}
            },
            Err(_) => fails.push(format!("{} - error", u)),
        }
        index = index + 1;
        let percent = (index * 100) / total;
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
