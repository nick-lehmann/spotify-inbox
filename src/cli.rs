use clap::{Parser, Subcommand};
extern crate xdg;

use crate::handler::SpotifyHandler;
use crate::APP_NAME;

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

    /// Show all of your playlists
    Playlist {},

    /// Show current inbox playlist
    Inbox {},

    /// Sync
    Sync {},

    /// Manage cache
    Config {},
}

pub fn run(handler: &SpotifyHandler) {
    let args = Cli::parse();

    match &args.command {
        Commands::Me {} => {
            handler.me();
        }
        Commands::Playlist {} => {
            handler.playlists();
        }
        Commands::Inbox {} => {
            handler.print_inbox();
        }
        Commands::Sync {} => {
            handler.sync();
        }
        Commands::Config {} => {
            // println!("Show config");

            // let config_file = xdg_dirs.find_config_file(config_name).unwrap();
            // println!("Config file can be found at: {:?}", config_file);
        }
    }
}
