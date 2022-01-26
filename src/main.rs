use clap::{Parser, Subcommand};
extern crate xdg;

mod commands;
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
    let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME).unwrap();
    let storage = storage::SpotifyStorage::new(&xdg_dirs);

    let client = spotify::get_client(&storage.path_helper.get_cache_path());

    let handler = SpotifyHandler {
        client: &client,
        storage: &storage,
        xdg_dirs: &xdg_dirs,
    };

    let args = Cli::parse();

    match &args.command {
        Commands::Me {} => {
            handler.me();
        }
        Commands::Inbox {} => {
            let inbox = handler.playlist_find_inbox();
            println!(
                "Your inbox playlist is called {} and has {} tracks",
                inbox.name, inbox.tracks.total
            );
        }
        Commands::Sync {} => sync::sync(&handler),
        Commands::Config {} => {
            // println!("Show config");

            // let config_file = xdg_dirs.find_config_file(config_name).unwrap();
            // println!("Config file can be found at: {:?}", config_file);
        }
    }
}
