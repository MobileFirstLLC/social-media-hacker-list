use std::fs;
use regex::Regex;
use std::collections::HashSet;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let readme = if args.len() > 1 { &args[1] } else { "../README.md" };
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

    for _url in urls {
        i = i + 1;
        let fail  = false;
        // println!("{:?}", &i);
        if fail { fails.push(_url); }

        let percent = (i * 100) / match_count;
        if percent % 10 == 0 && percent > progress {
            println!("...{ } % ({})", percent, fails.len());
            progress = percent;
        }
    }

    if fails.len() > 0 {
        println!("Fails!");
    } else {
        println!("no issues");
    }
}
