use clap::Parser;
use tui::start_tui;
use url::Url;
use worker::spawn_worker;

mod state;
mod tui;
mod worker;

#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(about = "Client for Pokerkees")]
struct Cli {
    #[arg(short, long, default_value_t = String::new())]
    #[arg(help = "The url of the Pokerkees server, include port is necessary")]
    url: String,

    #[arg(short, long, default_value_t = String::new())]
    #[arg(help = "The name to use for your Pokerkees user")]
    name: String,

    #[arg(short='o', long, default_value_t = String::new())]
    #[arg(help = "The org name you use in devops")]
    azure_organisation: String,

    #[arg(short='p', long, default_value_t = String::new())]
    #[arg(help = "The project name you use in devops within the org")]
    azure_project: String,
}

fn main() {
    // init logging
    // We will stop logging as soon as the TUI is up and running
    colog::init();
    log::info!("Booting Pokerkees client");
    // Start by reading cli stuff
    let cli = Cli::parse();
    let mut name = String::new();
    if !cli.name.is_empty() {
        if cli.name.len() > 20 {
            log::warn!("Name from cli too long. Only 20 chars allowed");
        }
        log::info!("Using name from cli arguments");
        name = cli.name;
    }
    let mut url: Option<Url> = None;
    if !cli.url.is_empty() {
        if let Ok(parsed_url) = Url::parse(&cli.url) {
            url = Some(parsed_url);
            log::info!("Using url from cli arguments");
        } else {
            log::error!("Passed url is not valid. Please check.");
            return;
        }
    }

    let mut azure_org: Option<String> = None;
    if !cli.azure_organisation.is_empty() {
        // Check somewhat sloppily if we have
        // a full url or not
        if !cli.azure_organisation.starts_with("https:") {
            // If not full url, append to standard url
            azure_org = Some(format!("https://dev.azure.com/{}", cli.azure_organisation));
        } else {
            azure_org = Some(cli.azure_organisation);
        }
    }

    let mut azure_proj: Option<String> = None;
    if !cli.azure_project.is_empty() {
        // No checks, just set
        azure_proj = Some(cli.azure_project);
    }

    let (s, r, handle) = spawn_worker();

    let state = state::init_state(name, url, s, r, azure_org, azure_proj);

    let tui_exit = start_tui(state);

    match tui_exit {
        Ok(_) => log::info!("Tui Exited normally"),
        Err(err) => {
            log::error!("{err}");
            log::error!("Tui errored. Exiting.");
        }
    }

    // Note the worker stops in two cases
    // - when the tui exits and the event sender gets dropped
    // - when the worker times out (which should not happen in working tui)
    log::info!("Waiting for worker to finish");
    match handle.join() {
        Ok(_) => log::info!("Worker finished"),
        Err(e) => log::error!("Error while closing worker: {e:?}"),
    }
}
