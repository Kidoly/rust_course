use reqwest;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use url::Url;
use percent_encoding::percent_decode_str;
use rocksdb::{DB, Options, IteratorMode};
use futures::stream::{FuturesUnordered, StreamExt};

/// Cleans a URL and returns a canonical URL if it starts with
/// "https://fr.wikipedia.org/wiki/", decodes percent‑encoded parts (except for %27),
/// and filters out URLs whose decoded title contains a colon.
fn clean_url(url_str: &str) -> Option<String> {
    let url_str = url_str.trim();

    // Skip pure fragments.
    if url_str.starts_with('#') {
        return None;
    }

    // Convert protocol-relative URLs to full HTTPS URLs.
    let url_str = if url_str.starts_with("//") {
        format!("https:{}", url_str)
    } else {
        url_str.to_string()
    };

    // Build a candidate canonical URL that should start with "https://fr.wikipedia.org/wiki/"
    let candidate: Option<String> = if url_str.starts_with('/') && !url_str.contains("://") {
        // Handle relative URLs.
        if url_str.starts_with("/w/index.php") {
            // Extract the "title" parameter.
            if let Some(pos) = url_str.find("title=") {
                let remainder = &url_str[pos + 6..];
                let title: String = remainder.chars().take_while(|&c| c != '&').collect();
                Some(format!("https://fr.wikipedia.org/wiki/{}", title))
            } else {
                None
            }
        } else if url_str.starts_with("/wiki/") {
            Some(format!("https://fr.wikipedia.org{}", url_str))
        } else {
            None
        }
    } else {
        // Handle absolute URLs.
        match Url::parse(&url_str) {
            Ok(mut parsed_url) => {
                // Only allow HTTPS.
                if parsed_url.scheme() != "https" {
                    return None;
                }
                // Only allow the French Wikipedia domain.
                if let Some(host) = parsed_url.host_str() {
                    if host != "fr.wikipedia.org" {
                        return None;
                    }
                } else {
                    return None;
                }
                // Convert /w/index.php URLs to canonical /wiki/ form.
                if parsed_url.path() == "/w/index.php" {
                    if let Some(title) = parsed_url.query_pairs()
                                                   .find(|(k, _)| k == "title")
                                                   .map(|(_, v)| v.into_owned()) {
                        Some(format!("https://fr.wikipedia.org/wiki/{}", title))
                    } else {
                        None
                    }
                } else if parsed_url.path().starts_with("/wiki/") {
                    // Remove any query parameters and fragments.
                    parsed_url.set_query(None);
                    parsed_url.set_fragment(None);
                    Some(parsed_url.to_string())
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    };

    // If we have a candidate, decode its title part and filter out URLs with a colon.
    if let Some(candidate) = candidate {
        let prefix = "https://fr.wikipedia.org/wiki/";
        if candidate.starts_with(prefix) {
            let title_encoded = &candidate[prefix.len()..];
            // Use a placeholder for %27 so that it remains intact.
            let placeholder = "___APOSTROPHE___";
            let title_with_placeholder = title_encoded.replace("%27", placeholder);
            let decoded_with_placeholder = percent_decode_str(&title_with_placeholder)
                .decode_utf8_lossy();
            let final_title = decoded_with_placeholder.replace(placeholder, "%27");
            // Skip URLs where the decoded title contains a colon.
            if final_title.contains(':') {
                return None;
            }
            return Some(format!("{}{}", prefix, final_title));
        }
        None
    } else {
        None
    }
}

/// Process a single URL:
/// 1. Downloads the page.
/// 2. Saves the HTML in the /wiki/ folder.
/// 3. Parses the page for new links and inserts them into the DB (if not already present).
/// 4. Marks the URL as processed (flag "1").
async fn process_url(db: Arc<DB>, url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Processing URL: {}", url);

    let response = reqwest::get(&url).await?;
    let page_body = response.text().await?;

    // Save the downloaded page in the "wiki" folder.
    let folder = "wiki";
    if !Path::new(folder).exists() {
        fs::create_dir_all(folder)?;
    }
    let filename = {
        let title = url.rsplit('/').next().unwrap_or("page");
        format!("{}/{}.html", folder, title)
    };
    fs::write(&filename, &page_body)?;
    println!("Saved page to '{}'.", filename);

    // Parse the downloaded page and extract links.
    let document = Html::parse_document(&page_body);
    let selector = Selector::parse("a").unwrap();
    let mut new_links: HashSet<String> = HashSet::new();
    for element in document.select(&selector) {
        if let Some(link) = element.value().attr("href") {
            if let Some(cleaned) = clean_url(link) {
                new_links.insert(cleaned);
            }
        }
    }

    // Insert any new links into the DB if they are not already present.
    for link in new_links {
        if db.get(link.as_bytes())?.is_none() {
            db.put(link.as_bytes(), b"0")?;
        }
    }

    // Mark the current URL as processed ("1") in the database.
    db.put(url.as_bytes(), b"1")?;
    println!("Marked URL '{}' as processed.", url);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "rocksdb_urls";
    let seed_url = "https://fr.wikipedia.org/wiki/Science";
    let mut opts = Options::default();
    opts.create_if_missing(true);

    let db: DB;
    // Check if the DB exists and if it has any unprocessed URL.
    if Path::new(db_path).exists() {
        let existing_db = DB::open(&opts, db_path)?;
        let mut has_unprocessed = false;
        for item in existing_db.iterator(IteratorMode::Start) {
            let (key, value) = item?;
            if value.as_ref() == b"0" {
                has_unprocessed = true;
                break;
            }
        }
        if has_unprocessed {
            println!("Existing DB has unprocessed URLs. Using existing DB.");
            db = existing_db;
        } else {
            println!("Existing DB found, but no unprocessed URLs. Clearing DB and seeding.");
            DB::destroy(&opts, db_path)?;
            db = DB::open(&opts, db_path)?;
            db.put(seed_url.as_bytes(), b"0")?;
            println!("Seed URL inserted: {}", seed_url);
        }
    } else {
        println!("No existing DB found. Creating new DB and seeding.");
        db = DB::open(&opts, db_path)?;
        db.put(seed_url.as_bytes(), b"0")?;
        println!("Seed URL inserted: {}", seed_url);
    }

    let db = Arc::new(db);

    // Main crawling loop.
    loop {
        let mut unprocessed_urls = Vec::new();
        for item in db.iterator(IteratorMode::Start) {
            let (key, value) = item?;
            if value.as_ref() == b"0" {
                unprocessed_urls.push(String::from_utf8(key.to_vec())?);
            }
        }

        if unprocessed_urls.is_empty() {
            println!("No unprocessed URLs left. Exiting.");
            break;
        }

        println!("Found {} unprocessed URLs. Processing concurrently...", unprocessed_urls.len());
        let mut futures = FuturesUnordered::new();
        for url in unprocessed_urls {
            let db_clone = Arc::clone(&db);
            futures.push(tokio::spawn(process_url(db_clone, url)));
        }
        while let Some(result) = futures.next().await {
            if let Err(e) = result {
                eprintln!("Task failed: {}", e);
            }
        }
    }

    Ok(())
}
