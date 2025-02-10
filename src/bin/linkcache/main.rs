use alfrusco::{config, Item, Runnable, URLItem, Workflow};
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
                let subtitle = link.subtitle.unwrap_or_default();
                item = item.subtitle(&subtitle);
                item = item.matches(format!("{} / {}", subtitle, &link.title));
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
