use std::collections::HashSet;

use console::style;
use rspotify_model::Id;
extern crate xdg;

use crate::handler::SpotifyHandler;
use crate::spotify::SpotifyHelper;

/// Flow:
/// - Get saved songs
/// - Get all playlists, only keep playlists created by the current user
/// - Get all songs in user playlists
/// - Compare songs
/// - Get songs in inbox
/// - Diff; add & remove songs
pub fn sync(handler: &SpotifyHandler) {
    println!("{} Fetching saved songs...", style("[1/4]").bold().dim());
    let saved_songs_vec = handler.saved_songs();
    let saved_songs: HashSet<String> = HashSet::from_iter(
        saved_songs_vec
            .iter()
            .map(|song| song.track.id.as_ref().unwrap().id().to_owned()),
    );

    println!("{} Fetching playlists...", style("[2/4]").bold().dim());
    let user_playlists = handler.playlist_get_owned_by_user();
    let tracks_in_playlists = handler.get_track_ids_in_playlists(&user_playlists);

    println!("{} Fetching inbox tracks...", style("[3/4]").bold().dim());
    let inbox_playlist = handler.playlist_find_inbox();
    let tracks_in_inbox = handler.get_track_ids_in_playlists(vec![inbox_playlist.clone()].as_ref());

    let unsorted_tracks: HashSet<String> = saved_songs
        .difference(&tracks_in_playlists)
        .cloned()
        .collect();

    let to_be_added: HashSet<String> = unsorted_tracks
        .difference(&tracks_in_inbox)
        .cloned()
        .collect();

    let to_be_removed: HashSet<String> = tracks_in_inbox
        .intersection(&tracks_in_playlists)
        .cloned()
        .collect();

    let additions = format!("{} ++", to_be_added.len());
    let removals = format!("{} --", to_be_removed.len());
    println!("Summary:");
    println!("========");
    println!(" - saved songs: {}", saved_songs.len());
    println!(" - unsorted songs: {}", unsorted_tracks.len());
    println!(
        " - changes to inbox: {} / {}",
        style(additions).bold().green(),
        style(removals).bold().red()
    );

    handler
        .client
        .playlist_add_all_items(&inbox_playlist.id, &to_be_added);
    handler
        .client
        .playlist_remove_all_items(&inbox_playlist.id, &to_be_removed)
}
