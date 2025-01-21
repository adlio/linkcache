use alfrusco::{config, Item, Runnable, URLItem, Workflow};
use clap::Parser;
use linkcache::{firefox, Cache};
use log::info;

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
}

fn main() {
    env_logger::init();
    let command = LinkCacheCLI::parse();

    alfrusco::execute(&config::AlfredEnvProvider, command, &mut std::io::stdout());
}

impl Runnable for LinkCacheCLI {
    type Error = WorkflowError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        info!("linkcache starting up");

        let query = self.query.join(" ").trim().to_string();

        let browser = firefox::Browser::new()?;
        let links = browser.search_bookmarks_directly(&query)?;

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
        workflow.response.append_items(items);

        // Allow Alfrusco to sort and filter the response
        workflow.set_filter_keyword(query.clone());

        Ok(())
    }
}
