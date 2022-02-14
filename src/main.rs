use clap::{Parser, Subcommand};
use commands::inbox::choose_inbox;
use config::SpotifyInboxConfig;
extern crate xdg;

mod commands;
mod config;
mod handler;
mod spotify;
mod storage;

use crate::commands::sync;
use crate::handler::SpotifyHandler;

const APP_NAME: &str = "spotify-inbox";

#[derive(Parser)]
#[clap(name = APP_NAME, about = "Create & organise your spotify inbox")]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Me
    Me {},

    /// Show current inbox playlist
    Inbox {},

    /// Sync
    Sync {},

    /// Manage cache
    Config {},
}

pub fn main() {
    let config = SpotifyInboxConfig::new(xdg::BaseDirectories::with_prefix(APP_NAME).unwrap());

    let storage = storage::SpotifyStorage::new(&config);

    let client = spotify::get_client(&storage.config.get_cache_path());

    let handler = SpotifyHandler {
        client: &client,
        storage: &storage,
    };

    let args = Cli::parse();

    match &args.command {
        Commands::Me {} => {
            handler.me();
        }
        Commands::Inbox {} => {
            choose_inbox(&handler, &config);
        }
        Commands::Sync {} => sync::sync(&handler),
        Commands::Config {} => {
            // println!("Show config");

            // let config_file = xdg_dirs.find_config_file(config_name).unwrap();
            // println!("Config file can be found at: {:?}", config_file);
        }
    }
}
