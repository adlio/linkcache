use alfrusco::{config, Item, Runnable, URLItem, Workflow};
use clap::Parser;
use linkcache::{arc, Cache};
use log::{info};
use std::process::Command;
use std::time::Duration;

mod error;

use error::WorkflowError;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author = "Aaron Longwell <aaron@adl.io>")]
#[command(version = "0.1.0")]
#[command(about = "Linkcache Utility")]
#[command(version, about, long_about = None)]
struct LinkCacheCLI {
    query: Vec<String>,

    #[clap(long, env = "UPDATE_ARC_CACHE", default_value = "false")]
    update_arc_cache: bool,
}

fn main() {
    env_logger::init();
    let command = LinkCacheCLI::parse();

    if command.update_arc_cache {
        let mut cache = Cache::default().expect("Could not create cache");
        let arc = arc::Browser::new();
        let links = arc
            .sidebar_links()
            .expect("Could not get Arc sidebar links");
        for link in links {
            cache
                .add(link.clone())
                .expect("Could not insert link into cache");
        }
        return;
    }

    alfrusco::execute(&config::AlfredEnvProvider, command, &mut std::io::stdout());
}

impl Runnable for LinkCacheCLI {
    type Error = WorkflowError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        info!("linkcache starting up");

        let query = self.query.join(" ").trim().to_string();
        // Update teh Arc browser cache in the background every 90 minutes
        let exe = std::env::current_exe()?;
        let mut cmd = Command::new(exe);
        cmd.arg("--update-arc-cache");
        workflow.run_in_background("update-arc-cache", Duration::from_secs(10), cmd);

        let cache = Cache::default()?;
        let results = cache.search(&query)?;
        info!("Found {} results from linkcache", results.len());

        let items: Vec<Item> = results
            .into_iter()
            .map(|link| {
                let mut item: Item = URLItem::new(&link.title, &link.url).into();
                let subtitle = link.subtitle.unwrap_or_default();
                item = item.subtitle(&subtitle);
                item = item.matches(format!("{} / {}", subtitle, &link.title));
                item
            })
            .collect();

        workflow.response.append_items(items);

        // Allow Alfrusco to sort and filter the response
        workflow.set_filter_keyword(query.clone());

        Ok(())
    }
}
