use chrono::offset::Utc;
use chrono::DateTime;
use regex::Regex;
use reqwest::{Client, Error, StatusCode};
use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use url::{Host, Url};

const INACTIVE_AFTER_N_DAYS: i64 = 365;
const ACCEPT_STATUS: [u16; 3] = [200, 403, 406];

enum CheckResult {
    Success(),
    Error(String),
}

#[derive(Deserialize)]
struct Repo {
    pushed_at: DateTime<Utc>,
    archived: bool,
}

fn to_url(url: &str) -> Url {
    Url::parse(url).expect("Invalid url")
}

fn is_ok(status: StatusCode, url: &str) -> bool {
    let su16 = status.as_u16();
    ACCEPT_STATUS.iter().find(|&&x| x == su16).is_some()
        // the acceptable edge cases
        || (status == 502 && url.contains("reddit.com/"))
        || (status == 429 && url.contains("apps.apple.com/"))
}

fn is_repo(url: &str) -> bool {
    let res = to_url(url);
    res.host().is_some()
        && res.host().unwrap() == Host::Domain("github.com")
        && res.path().split("/").count() > 2
}

fn construct_api_url(url: &str) -> String {
    let res = to_url(url);
    let parts: Vec<&str> = res.path().split("/").collect();
    format!("https://api.github.com/repos/{}/{}", parts[1], parts[2])
}

fn is_inactive(date: DateTime<Utc>) -> bool {
    Utc::now().signed_duration_since(date).num_days() >= INACTIVE_AFTER_N_DAYS
}

async fn check_repo(u: &str, token: &str) -> Result<CheckResult, Error> {
    let response = Client::new()
        .get(&construct_api_url(u))
        .header("User-Agent", "url-checker")
        .header("Accept", "application/vnd.github.v3+json")
        .header("Authorization", format!("token {}", token))
        .send()
        .await?;
    let status = response.status();
    if is_ok(status, u) {
        let repo: Repo = response.json().await?;
        let mut result = CheckResult::Success();
        if repo.archived {
            result = CheckResult::Error(String::from("archived"));
        }
        if is_inactive(repo.pushed_at) {
            result = CheckResult::Error(String::from("inactive"));
        }
        return Ok(result);
    }
    Ok(CheckResult::Error(status.to_string()))
}

async fn check_url(u: &str) -> Result<CheckResult, Error> {
    let head = Client::new()
        .head(u)
        .header("Accept", "*/*")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
            (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 Edg/134.0.0.0",
        )
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
    Ok(CheckResult::Error(get.status().to_string()))
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let input_file = args.get(1).map(|s| s.as_str()).unwrap_or("README.md");
    let token = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let pattern = r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.([a-zA-Z0-9()]){1,6}\b([-a-zA-Z0-9@:%_+.~#?&/=]*)";
    let contents = read_to_string(input_file).expect("File read error!");
    let re = Regex::new(pattern).unwrap();

    let urls: Vec<&str> = re
        .find_iter(&contents)
        .filter_map(|u| Some(u.as_str()))
        .collect::<HashSet<&str>>()
        .into_iter()
        .collect();

    let total = urls.len();
    let mut fails: Vec<String> = Vec::new();
    let mut progress = 0;
    let mut index = 0;
    println!("Checking {total} entries");

    for u in urls {
        let check_result = if is_repo(u) {
            check_repo(u, token).await
        } else {
            check_url(u).await
        };
        match check_result {
            Ok(CheckResult::Success()) => {}
            Ok(CheckResult::Error(reason)) => fails.push(format!("{u} - {reason}")),
            Err(_) => fails.push(format!("{u} - error")),
        }
        index += 1;
        let percent = (index * 100) / total;
        if percent % 10 == 0 && percent > progress {
            println!("...{percent} % ({})", fails.len());
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
    println!("✓ all checks passed");
}
