extern crate linkcache;

use alfrusco::{Item, URLItem, Workflow};
use clap::Parser;
use linkcache::{arc, Cache, Result};
use log::{error, info};
use std::process::Command;
use std::time::Duration;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    query: Vec<String>,

    #[clap(long, env = "UPDATE_ARC_CACHE", default_value = "false")]
    update_arc_cache: bool,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    if args.update_arc_cache {
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

    Workflow::from_env()
        .expect("Could not parse workflow config")
        .run(run);
}

fn run(wf: &mut Workflow) -> Result<()> {
    let args = Args::parse();
    let query = args.query.join(" ").trim().to_string();

    // Update teh Arc browser cache in the background every 90 minutes
    let exe = std::env::current_exe()?;
    let mut cmd = Command::new(exe);
    cmd.arg("--update-arc-cache");
    wf.run_in_background("update-arc-cache", Duration::from_secs(10), cmd);

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

    wf.response.append_items(items);

    // Allow Alfrusco to sort and filter the response
    wf.set_filter_keyword(query.clone());

    Ok(())
}
