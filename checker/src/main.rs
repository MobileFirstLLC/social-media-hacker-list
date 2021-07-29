use std::fs::read_to_string;
use std::env;
use regex::Regex;
use std::collections::HashSet;
use reqwest::{Error, Client, StatusCode};
use url::{Url, Host};
use serde::Deserialize;
use chrono::DateTime;
use chrono::offset::Utc;

#[derive(Deserialize)]
struct Repo { updated_at: DateTime<Utc> }

struct CheckResult { url: String, success: bool, reason: String }

impl CheckResult {
    fn new(url: &str) -> CheckResult {
        CheckResult { success: true, url: url.to_string(), reason: "".to_string() }
    }
    fn inactive(url: &str) -> CheckResult {
        CheckResult { success: false, url: url.to_string(), reason: "inactive".to_string() }
    }
    fn error(url: &str, err: &str) -> CheckResult {
        CheckResult { success: false, url: url.to_string(), reason: err.to_string() }
    }
}

fn is_ok(status: StatusCode, url: &str) -> bool {
    return status == 200 || status == 403 || status == 406 ||
        (status == 502 && url.contains("reddit.com/"));
}

fn get_url(u: &str) -> Url {
    return Url::parse(u).expect("Invalid url");
}

fn is_repo(u: &str) -> bool {
    let res = get_url(u);
    return res.host() == Some(Host::Domain("github.com")) && res.path().split("/").count() > 2;
}

fn gh_repo_url(u: &str) -> String {
    let res = get_url(u);
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
        .send().await?;
    let status = response.status();
    if is_ok(status, u) {
        let repo: Repo = response.json().await?;
        if is_active(repo.updated_at) {
            return Ok(CheckResult::new(u));
        }
        return Ok(CheckResult::inactive(u));
    }
    return Ok(CheckResult::error(u, &status.to_string()));
}

async fn check_url(u: &str) -> Result<CheckResult, Error> {
    let head = Client::new().head(u)
        .header("Accept", "*/*")
        .header("User-Agent", "url-checker")
        .send().await?;
    if is_ok(head.status(), u) {
        return Ok(CheckResult::new(u));
    }
    let get = Client::new().get(u)
        .header("Accept", "*/*")
        .header("User-Agent", "url-checker")
        .send().await?;
    if is_ok(get.status(), u) {
        return Ok(CheckResult::new(u));
    }
    let text = get.text().await?;
    return Ok(CheckResult::error(u, &text));
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let readme = if args.len() > 1 { &args[1] } else { "../README.md" };
    let token = if args.len() > 2 { &args[2] } else { "" };
    let pattern = r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.([a-zA-Z0-9()]){1,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)";
    let contents = read_to_string(readme).expect("File read error!");
    let re = Regex::new(pattern).unwrap();
    let matches: Vec<&str> = re.find_iter(&contents).filter_map(|cap| { Some(cap.as_str()) }).collect();
    let unique_urls: HashSet<_> = matches.iter().cloned().collect();
    let urls: Vec<_> = unique_urls.into_iter().collect();
    let match_count = urls.len();
    let mut fails: Vec<CheckResult> = Vec::new();
    let mut progress = 0;
    let mut i = 0;

    println!("Checking {} entries...", match_count);
    for u in urls {
        let check_result;
        if is_repo(u) {
            check_result = check_repo(u, token).await;
        } else {
            check_result = check_url(u).await;
        }
        match check_result {
            Ok(res) => { if !res.success { fails.push(res) } }
            Err(_) => { fails.push(CheckResult::error(u, "error")) }
        }
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
            println!("- {} {}", fail.reason, fail.url);
        }
        std::process::exit(1);
    }
    println!("no issues");
}