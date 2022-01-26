use std::collections::HashSet;

use clap::{Parser, Subcommand};
use rspotify_model::Id;
extern crate xdg;

use crate::handler::SpotifyHandler;
use crate::spotify::SpotifyHelper;
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

    /// Show current inbox playlist
    Inbox {},

    /// Sync
    Sync {},

    /// Manage cache
    Config {},
}

/// Flow:
/// - Get saved songs
/// - Get all playlists, only keep playlists created by the current user
/// - Get all songs in user playlists
/// - Compare songs
/// - Get songs in inbox
/// - Diff; add & remove songs
pub fn sync(handler: &SpotifyHandler) {
    let saved_songs_vec = handler.saved_songs();
    let saved_songs: HashSet<String> = HashSet::from_iter(
        saved_songs_vec
            .iter()
            .map(|song| song.track.id.as_ref().unwrap().id().to_owned()),
    );

    let user_playlists = handler.playlist_get_owned_by_user();
    let tracks_in_playlists = handler.get_track_ids_in_playlists(&user_playlists);

    let unsorted_tracks: HashSet<String> = saved_songs
        .difference(&tracks_in_playlists)
        .cloned()
        .collect();

    println!("You have {} saved songs", saved_songs.len());
    println!("You have {} unsorted songs", unsorted_tracks.len());

    let inbox_playlist = handler.playlist_find_inbox();
    let tracks_in_inbox = handler.get_track_ids_in_playlists(vec![inbox_playlist.clone()].as_ref());

    let to_be_added: HashSet<String> = unsorted_tracks
        .difference(&tracks_in_inbox)
        .cloned()
        .collect();

    let to_be_removed: HashSet<String> = tracks_in_inbox
        .intersection(&tracks_in_playlists)
        .cloned()
        .collect();

    println!("{} songs will be added to your inbox", to_be_added.len());
    println!(
        "{} songs will be removed from your inbox",
        to_be_removed.len()
    );

    handler
        .client
        .playlist_add_all_items(&inbox_playlist.id, &to_be_added);
    handler
        .client
        .playlist_remove_all_items(&inbox_playlist.id, &to_be_removed)
}

pub fn run(handler: &SpotifyHandler) {
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
        Commands::Sync {} => sync(handler),
        Commands::Config {} => {
            // println!("Show config");

            // let config_file = xdg_dirs.find_config_file(config_name).unwrap();
            // println!("Config file can be found at: {:?}", config_file);
        }
    }
}
