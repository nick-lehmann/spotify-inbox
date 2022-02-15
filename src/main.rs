use clap::{Parser, Subcommand};
use commands::inbox::choose_inbox;
use config::SpotifyInboxConfig;
extern crate xdg;

mod commands;
mod config;
mod handler;
mod spotify;
mod storage;

use commands::sync;
use handler::SpotifyHandler;

const APP_NAME: &str = "spotify-inbox";

#[derive(Parser)]
#[clap(name = APP_NAME, about = "Create & organise your spotify inbox")]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set your inbox playlist
    Inbox {},

    /// Synchronise your inbox
    Sync {},
}

pub fn main() {
    let config = SpotifyInboxConfig::new(xdg::BaseDirectories::with_prefix(APP_NAME).unwrap());

    let storage = storage::SpotifyStorage::new(&config);

    let cache_path = &storage.config.get_cache_path();
    println!("Cache path: {}", cache_path.display());
    let client = spotify::get_client(&cache_path);

    let handler = SpotifyHandler {
        client: &client,
        storage: &storage,
    };

    let args = Cli::parse();

    match &args.command {
        Commands::Inbox {} => {
            choose_inbox(&handler, &config);
        }
        Commands::Sync {} => sync::sync(&handler),
    }
}
