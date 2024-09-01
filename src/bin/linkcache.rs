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

    let _cache = Cache::default()?;
    let arc = arc::Browser::new();
    let links = arc.sidebar_links()?;
    let items: Vec<Item> = links
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

    Ok(())
}
