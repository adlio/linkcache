use alfrusco::{config, Item, Runnable, URLItem, Workflow, ICON_BOOKMARK, ICON_CLOCK};
use clap::Parser;
use linkcache::{firefox, Cache};
use log::{error, info};
use std::env;
use std::process::Command;
use std::time::Duration;

mod error;

use error::WorkflowError;

const MAX_FIREFOX_AGE_IN_MINS: u64 = 2;

#[derive(Parser, Debug)]
#[command(author = "Aaron Longwell <aaron@adl.io>")]
#[command(version = "0.1.0")]
#[command(about = "Alfred workflow to ")]
#[command(version, about, long_about = None)]
struct LinkCacheCLI {
    #[clap(short, long, env)]
    cache: bool,

    query: Vec<String>,
}

fn main() {
    env_logger::init();
    let command = LinkCacheCLI::parse();

    if command.cache {
        match update_cache() {
            Ok(_) => {
                return;
            }
            Err(e) => {
                error!("Error updating cache: {}", e);
                return;
            }
        }
    }

    alfrusco::execute(&config::AlfredEnvProvider, command, &mut std::io::stdout());
}

impl Runnable for LinkCacheCLI {
    type Error = WorkflowError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        info!("linkcache starting up");

        workflow.run_in_background(
            "firefox-update",
            Duration::from_secs(60 * MAX_FIREFOX_AGE_IN_MINS),
            firefox_update_cmd(),
        );

        let query = self.query.join(" ").trim().to_string();

        let cache = Cache::new()?;
        let items: Vec<Item> = cache
            .search(&query)?
            .into_iter()
            .map(|link| {
                let mut item: Item = URLItem::new(&link.title, &link.url).into();

                // Strip protocol from URL
                let url = link
                    .url
                    .strip_prefix("https://")
                    .or_else(|| link.url.strip_prefix("http://"))
                    .unwrap_or(&link.url);

                // Build subtitle based on source type
                let (subtitle, icon, boost) = if link.is_bookmark() {
                    let folder = link.subtitle.as_deref().unwrap_or_default();
                    let subtitle = format_bookmark_subtitle(folder, url);
                    (subtitle, ICON_BOOKMARK, 100)
                } else {
                    (url.to_string(), ICON_CLOCK, 0)
                };

                item = item.subtitle(&subtitle);
                item = item.matches(format!("{} / {}", subtitle, &link.title));
                item = item.icon_from_image(icon).boost(boost);

                item
            })
            .collect();
        info!("Found {} matching results in cache", items.len());
        workflow.response.append_items(items);

        // Allow Alfrusco to sort and filter the response
        workflow.set_filter_keyword(query.clone());

        Ok(())
    }
}

fn update_cache() -> Result<(), WorkflowError> {
    let mut cache = Cache::new()?;
    let browser = firefox::Browser::new()?;
    browser.create_places_replica(&cache)?;
    browser.cache_bookmarks(&mut cache)?;
    browser.cache_history(&mut cache)?;
    Ok(())
}

/// Format a bookmark subtitle with folder path and URL.
/// Uses fish-style shortening for long folder paths.
fn format_bookmark_subtitle(folder: &str, url: &str) -> String {
    const MAX_LEN: usize = 80;
    const SEPARATOR: &str = " · ";

    if folder.is_empty() {
        return url.to_string();
    }

    let full = format!("{}{}{}", folder, SEPARATOR, url);
    if full.len() <= MAX_LEN {
        return full;
    }

    // Try fish-style shortening: "Work / Areas / Alfred" -> "W / A / Alfred"
    let shortened_folder = shorten_folder_path_fish_style(folder);
    let shortened = format!("{}{}{}", shortened_folder, SEPARATOR, url);

    if shortened.len() <= MAX_LEN {
        shortened
    } else {
        // Still too long, truncate URL
        let available = MAX_LEN.saturating_sub(shortened_folder.len() + SEPARATOR.len() + 1);
        if available > 10 {
            format!("{}{}{}…", shortened_folder, SEPARATOR, &url[..available])
        } else {
            // Just show URL truncated
            format!("{}…", &url[..MAX_LEN.saturating_sub(1).min(url.len())])
        }
    }
}

/// Shorten folder path fish-style: "Work / Areas / Alfred" -> "W / A / Alfred"
/// Keeps the last segment full, abbreviates earlier segments to first char.
fn shorten_folder_path_fish_style(path: &str) -> String {
    let parts: Vec<&str> = path.split(" / ").collect();
    if parts.len() <= 1 {
        return path.to_string();
    }

    let mut result = Vec::with_capacity(parts.len());
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Keep last segment full
            result.push(part.to_string());
        } else {
            // Abbreviate to first character
            result.push(
                part.chars()
                    .next()
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
            );
        }
    }
    result.join(" / ")
}

/// TODO This could be made more generic with improvements to
/// alfrusco.
///
fn firefox_update_cmd() -> Command {
    let mut cmd = Command::new(env::current_exe().expect("Couldn't determine current executable"));

    cmd.args(vec!["--cache"]);

    // Set the current working directory
    if let Ok(current_dir) = env::current_dir() {
        cmd.current_dir(current_dir);
    }

    // Set all environment variables
    for (key, value) in env::vars() {
        cmd.env(key, value);
    }

    cmd
}
