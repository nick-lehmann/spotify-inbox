use std::collections::HashSet;

use console::style;
use indicatif::ProgressBar;
use rspotify::clients::OAuthClient;
use rspotify_model::{Id, Page, SavedTrack};
extern crate xdg;

use crate::handler::SpotifyHandler;
use crate::spotify::SpotifyHelper;

pub fn sync(handler: &SpotifyHandler) {
    // STEP 1: Get saved songs
    // =======================
    // TODO: Don't fetch first page for nothing
    let page: Page<SavedTrack> = handler
        .client
        .current_user_saved_tracks_manual(None, Some(50), Some(0))
        .unwrap();

    let saved_songs_total = page.total;

    println!("{} Fetching saved songs...", style("[1/4]").bold().dim());
    let pb = ProgressBar::new(saved_songs_total as u64);

    let mut saved_songs: Vec<SavedTrack> = Vec::new();
    for saved_song in handler.client.current_user_saved_tracks(None) {
        saved_songs.push(saved_song.unwrap());
        pb.inc(1);
    }

    pb.finish_and_clear();

    let saved_songs_set: HashSet<String> = HashSet::from_iter(
        saved_songs
            .iter()
            .map(|song| song.track.id.as_ref().unwrap().id().to_owned()),
    );

    // STEP 2: Get user playlists
    // ==========================
    println!("{} Fetching playlists...", style("[2/4]").bold().dim());
    let user_playlists = handler.playlist_get_owned_by_user();
    let tracks_in_playlists = handler.get_track_ids_in_playlists(&user_playlists);

    // STEP 3: Fetch all songs currently in inbox
    // ==========================================
    println!("{} Fetching inbox tracks...", style("[3/4]").bold().dim());
    let inbox_playlist = handler.playlist_find_inbox();
    let tracks_in_inbox = handler.get_track_ids_in_playlists(vec![inbox_playlist.clone()].as_ref());

    // STEP 4: Calculate diffs between songs
    // =====================================
    let unsorted_tracks: HashSet<String> = saved_songs_set
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

    // STEP 5: Apply changes to inbox
    // ==============================
    handler
        .client
        .playlist_add_all_items(&inbox_playlist.id, &to_be_added);
    handler
        .client
        .playlist_remove_all_items(&inbox_playlist.id, &to_be_removed)
}
