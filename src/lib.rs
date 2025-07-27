use std::{
    fs::{File, OpenOptions},
    io::Read,
    io::Write,
    path::PathBuf,
    process::Command,
};

pub const NO_QUOTE: &str = "<NO QUOTE>";
pub const NEWLINE_REPLACEMENT: &str = ";;";

struct TwirUrl {
    url: String,
    date: String,
}

pub struct Quote {
    pub date: String,
    pub text: String,
}

fn get_storage_file() -> File {
    let mut data_home = std::env::var("XDG_DATA_HOME").unwrap_or_default();

    if data_home.trim().is_empty() {
        let home = std::env::var("HOME").expect("Environement variable $HOME is not set");
        data_home = format!("{}/.local/share", home);
    }

    let data_home = PathBuf::from(data_home).join("qwot");

    std::fs::create_dir_all(&data_home).expect("Failed to create storage directory");

    let store_file = data_home.join("quotes");
    OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&store_file)
        .expect("Failed to open storage file")
}

fn get_new_twir_urls() -> Option<Vec<TwirUrl>> {
    const TWIR_LIST_URL: &str = "https://this-week-in-rust.org/blog/archives/index.html";

    let data = Command::new("curl")
        .arg(TWIR_LIST_URL)
        .output()
        .expect("Failed to execute `curl` command");
    let data = String::from_utf8_lossy(&data.stdout).to_string();

    if data.is_empty() {
        return None;
    }

    let paste_issue = data.find("Past issues").unwrap();
    let data = &data[paste_issue..];

    let _02_mar_2015 = data.find("This Week in Rust 72").unwrap();
    let data = &data[.._02_mar_2015];

    let urls = data
        .split("<li>")
        .skip(1)
        .map(|li| {
            let datetime = li.find("datetime").unwrap();
            let li = &li[datetime + 10..]; // 10 is the length of `datetime="`

            let date = &li[..10];

            let href = li.find("href").unwrap();
            let li = &li[href + 6..];

            let end_url = li.find('"').unwrap();
            let url = &li[..end_url];

            TwirUrl {
                url: url.to_string(),
                date: date.to_string(),
            }
        })
        .collect();

    Some(urls)
}

#[test]
fn a() {
    let a =
        get_qotw("https://this-week-in-rust.org/blog/2015/03/16/this-week-in-rust-74/").unwrap();
    println!("\n{a}");
}

fn get_qotw(url: &str) -> Option<String> {
    let data = Command::new("curl")
        .arg(url)
        .output()
        .expect("Failed to execute `curl` command");
    let data = String::from_utf8_lossy(&data.stdout).to_string();

    if data.is_empty() {
        return None;
    }

    let qotw_title = data.find("Quote of the Week")?;
    let data = &data[qotw_title + 17 + 10..];

    let after_qotw = data.find("for next week")?;
    let data = &data[..after_qotw];

    let quote = if let Some(blockquote) = data.find("<blockquote>") {
        let data = &data[blockquote + 12..];

        let mut last_blockquote = data;
        let mut offset = 0;

        while let Some(blockquote) = last_blockquote.find("<blockquote>") {
            last_blockquote = &last_blockquote[blockquote + 12..];
            offset += blockquote + 12;
        }

        let end_quote = offset + last_blockquote.find("</blockquote>").unwrap();
        let quote = &data[..end_quote];

        Some(quote)
    } else if let Some(em) = data.find("<em>") {
        if url.contains("2015") {
            let end_quote = data.find("</em>").unwrap();
            let quote = &data[em + 4..end_quote];

            Some(quote)
        } else {
            None
        }
    } else if let Some(pre) = data.find("<pre>") {
        let end_quote = data.find("</pre>").unwrap();
        let quote = &data[pre + 5..end_quote];

        Some(quote)
    } else {
        dbg!(data);
        None
    };

    quote.map(|quote| {
        let mut quote = quote.to_string();

        let mut remove_surrounding_tag = |open: &str, close: &str, replace: &str| {
            quote = quote.replace(close, "");

            while let Some(link) = quote.find(open) {
                let end_link = link + quote[link..].find(">").unwrap() + 1;
                let pat = &quote[link..end_link];

                quote = quote.replace(pat, replace);
            }
        };

        remove_surrounding_tag("<a", "</a>", "");
        remove_surrounding_tag("<h1", "</h1>", "# ");
        remove_surrounding_tag("<span", "</span>", "");

        quote
            .replace("\\[", "[")
            .replace("\\]", "]")
            .replace("\\<", "<")
            .replace("\\>", ">")
            .replace("<blockquote>", "")
            .replace("</blockquote>", "")
            .replace("<p>", "")
            .replace("</p>", "")
            .replace("<ul>", "")
            .replace("</ul>", "")
            .replace("<ol>", "")
            .replace("</ol>", "")
            .replace("<li>", "- ")
            .replace("</li>", "")
            .replace("<em>", "_")
            .replace("</em>", "_")
            .replace("<strong>", "*")
            .replace("</strong>", "*")
            .replace("<code>", "`")
            .replace("</code>", "`")
            .replace("<br>", "")
            .replace("<hr>", "---")
            .replace("\\&lt;", "<")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&#39;", "'")
            .trim()
            .replace("\n", NEWLINE_REPLACEMENT)
    })
}

pub fn fetch_missing_quotes() -> Option<()> {
    let mut storage_file = get_storage_file();

    let mut quotes = String::new();
    storage_file.read_to_string(&mut quotes).ok();

    let fetched_quotes_date = quotes
        .lines()
        .map(|line| line.split(' ').next().unwrap())
        .collect::<Vec<_>>();

    let twir_urls = get_new_twir_urls()?;
    // println!("twir_url len: {}", twir_urls.len());

    for twir_url in twir_urls.into_iter().rev() {
        if fetched_quotes_date.contains(&twir_url.date.as_str()) {
            continue;
        }

        // println!("FETCH {}", twir_url.url);
        let qotw = get_qotw(&twir_url.url);
        if let Some(qotw) = qotw {
            writeln!(storage_file, "{} {}", twir_url.date, qotw).ok();
        } else {
            writeln!(storage_file, "{} {}", twir_url.date, NO_QUOTE).ok();
        }
    }

    Some(())
}

pub fn get_quotes() -> Vec<Quote> {
    let mut storage_file = get_storage_file();

    let mut quotes = String::new();
    storage_file.read_to_string(&mut quotes).ok();

    quotes
        .lines()
        .map(|line| {
            let data = line.split_once(' ').unwrap();
            Quote {
                date: data.0.to_owned(),
                text: data.1.to_owned(),
            }
        })
        .collect()
}
