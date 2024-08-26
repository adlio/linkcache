extern crate linkcache;

use alfrusco::{Item, URLItem, Workflow, WorkflowConfig};
use clap::Parser;
use linkcache::{arc, Cache};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    query: Vec<String>,
}

fn main() {
    let config = WorkflowConfig::for_testing().expect("Failed to create alfrusco config");
    Workflow::run(config, run);
}

fn run(wf: &mut Workflow) -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let cache = Cache::default()?;
    let arc = arc::Browser::new();
    let links = arc.sidebar_links()?;
    let items: Vec<Item> = links
        .into_iter()
        .map(|link| Item::new(link.title).arg(link.url))
        .collect();
    wf.response.append_items(items);

    Ok(())
}
