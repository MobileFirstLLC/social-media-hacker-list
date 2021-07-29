use std::fs;
use regex::Regex;
use std::collections::HashSet;
use std::env;

fn is_repo(url: &str) -> bool {
    return url.contains("github.com");
}

fn is_ok(status: reqwest::StatusCode, url: &str) -> bool {
    return status == 200 || status == 403 || status == 406 ||
        (status == 502 && url.contains("reddit.com"));
}

async fn check_repo(_url: &str) -> Result<bool, reqwest::Error> {
    return Ok(true);
}

async fn make_request(url: &str) -> Result<bool, reqwest::Error> {
    let client = reqwest::Client::builder().build()?;
    let mut response = client.head(url).send().await?;
    if is_ok(response.status(), url) { return Ok(true); }
    response = client.get(url).send().await?;
    if is_ok(response.status(), url) { return Ok(true); }
    Ok(false)
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

    for url in urls {
        let status_check =
            if is_repo(url) { check_repo(url).await } else { make_request(&url).await };

        match status_check {
            Ok(true) => {}
            Ok(false) => fails.push(url),
            Err(_e) => fails.push(url),
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
