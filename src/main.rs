use std::process::ExitCode;

use qwot::{NEWLINE_REPLACEMENT, NO_QUOTE, fetch_missing_quotes, get_quotes};

const HELP: &str = r#"qwot - Print a random twir's quote of the week

Usage: qwot [OPTIONS]

Options:
  -d <DATE>  Print the quote of the given date (YYYY-MM-DD)
  -f         Fetch last quotes of twir
  -h --help  Print this message"#;

fn main() -> ExitCode {
    let argv = std::env::args().collect::<Vec<String>>();

    let mut try_fetch_missing_quotes = false;
    let mut date = None;
    if let Some(arg) = argv.get(1) {
        match arg.as_str() {
            "-f" => try_fetch_missing_quotes = true,
            "-d" => {
                date = argv.get(2);
                if date.is_none() {
                    println!("No date provided");
                    return ExitCode::FAILURE;
                }
            }
            "-h" | "--help" => {
                println!("{HELP}");
                return ExitCode::SUCCESS;
            }
            _ => {}
        }
    }

    if try_fetch_missing_quotes {
        let new_quotes_count = fetch_missing_quotes().expect("Failed to fetch missing quotes");
        println!("Fetched {} new quotes", new_quotes_count);
        return ExitCode::SUCCESS;
    }

    if let Some(date) = date {
        let quotes = get_quotes();
        if let Some(quote) = quotes.iter().find(|q| q.date == *date) {
            if quote.text == NO_QUOTE {
                println!("No quote available for date: {}", date);
                return ExitCode::FAILURE;
            }
            println!(
                "{}\n\t\t- {} -",
                quote.text.replace(NEWLINE_REPLACEMENT, "\n").trim(),
                quote.date
            );
            return ExitCode::SUCCESS;
        } else {
            println!(
                "No quote found for date: {}. Try fetching new quotes with `qwot -f`",
                date
            );
            return ExitCode::FAILURE;
        }
    }

    let quotes = get_quotes();

    if quotes.is_empty() {
        println!("No quote available, try `qwot -f` to fetch new quotes");
        return ExitCode::SUCCESS;
    }

    let mut random_idx = random() % quotes.len();
    let mut quote = &quotes[random_idx];

    while quote.text == NO_QUOTE {
        random_idx = random() % quotes.len();
        quote = &quotes[random_idx];
    }

    println!(
        "{}\n\t\t- {} -",
        quote.text.replace(NEWLINE_REPLACEMENT, "\n").trim(),
        quote.date
    );

    ExitCode::SUCCESS
}

// Code from the fastrand crate
fn random() -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::thread;
    use std::time::Instant;

    let mut hasher = DefaultHasher::new();
    Instant::now().hash(&mut hasher);
    thread::current().id().hash(&mut hasher);

    hasher.finish() as usize
}
