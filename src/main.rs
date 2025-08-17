
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
    /// Print debugging info (status, headers, raw body snippet) on failures
    #[arg(long)]
    debug: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = real_main().await {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

/// Program entry after argument parsing.
async fn real_main() -> Result<()> {
    // Load .env file if present (ignores if missing)
    let _ = dotenv();
    let cli = Cli::parse();
    let slug = cli.slug;

    let api_key = env::var("LEANPUB_API_KEY")
        .context("LEANPUB_API_KEY env var must be set with your Leanpub API key")?;

    let client = Client::new();

    let url = format!("https://leanpub.com/{}.json?api_key={}", slug, api_key);
    let json = fetch_book(&client, &url, cli.debug).await?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// Fetch the Leanpub book configuration JSON from the endpoint
async fn fetch_book(client: &Client, url: &str, debug: bool) -> Result<serde_json::Value> {
    let resp = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .with_context(|| format!("sending request to {url}"))?;

    let status = resp.status();
    let headers_debug = if debug { Some(format!("{:?}", resp.headers())) } else { None };
    let bytes = resp.bytes().await.context("reading response body")?;

    if !status.is_success() {
        let mut msg = format!("request to {url} failed with {status}");
        if debug {
            let snippet = String::from_utf8_lossy(&bytes)
                .chars()
                .take(200)
                .collect::<String>();
            msg.push_str(&format!(" | body snippet: {:?}", snippet));
            if let Some(h) = headers_debug { msg.push_str(&format!(" | headers: {h}")); }
        }
        anyhow::bail!(msg);
    }

    let json: serde_json::Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing JSON from {url}"))?;
    Ok(json)
}
