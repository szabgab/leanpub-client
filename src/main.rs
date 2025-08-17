
use std::env;
use dotenvy::dotenv;
use clap::Parser;
use reqwest::Client;
use anyhow::{Context, Result};
// Leanpub API typically uses API key as a query parameter (?api_key=) for many endpoints.

/// Retrieves and prints all configuration options for a Leanpub book given its slug.
///
/// # Arguments
///
/// * `slug` - The slug of the Leanpub book.
/// * `api_key` - The Leanpub API key.
#[derive(Parser, Debug)]
#[command(name = "leanpub-client", version, about = "Leanpub Book Configuration Fetcher", author = "")]
struct Cli {
    /// Book slug (the part after https://leanpub.com/ in the URL)
    slug: String,
    /// Use the legacy /json endpoint instead of metadata
    #[arg(long)]
    legacy: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = real_main().await {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

async fn real_main() -> Result<()> {
    // Load .env file if present (ignores if missing)
    let _ = dotenv();
    let cli = Cli::parse();
    let slug = cli.slug;

    let api_key = env::var("LEANPUB_API_KEY")
        .context("LEANPUB_API_KEY env var must be set with your Leanpub API key")?;

    let client = Client::new();

    // Two possible endpoints: legacy public JSON, and authenticated API (example placeholder)
    // Public: https://leanpub.com/<slug>.json  (no api_key)
    // Auth (example for book metadata): https://leanpub.com/<slug>/book_metadata.json?api_key=...
    // If 404 on one, we can try the other.
    let mut tried = Vec::new();

    let endpoints = if cli.legacy {
        vec![format!("https://leanpub.com/{}.json", slug)]
    } else {
        vec![
            format!("https://leanpub.com/{}/book_metadata.json?api_key={}", slug, api_key),
            format!("https://leanpub.com/{}.json", slug),
        ]
    };

    for url in endpoints {
        let resp = client.get(&url).send().await;
        match resp {
            Ok(r) => {
                let status = r.status();
                if status.is_success() {
                    let bytes = r.bytes().await.context("reading response body")?;
                    let json: serde_json::Value = serde_json::from_slice(&bytes)
                        .context("parsing JSON")?;
                    println!("{}", serde_json::to_string_pretty(&json)?);
                    return Ok(());
                } else {
                    tried.push(format!("{} => {}", url, status));
                }
            }
            Err(err) => {
                tried.push(format!("{} => request error: {}", url, err));
            }
        }
    }

    anyhow::bail!("All endpoints failed:\n{}", tried.join("\n"));
}
