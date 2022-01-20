use std::{collections::HashSet, fs};

use rspotify::{
    clients::{BaseClient, OAuthClient},
    AuthCodePkceSpotify,
};
use rspotify_model::{
    Id, PlayableId, PlayableItem, PlaylistId, PlaylistItem, PrivateUser, SavedTrack,
    SimplifiedPlaylist, TrackId,
};

use crate::storage;

pub struct SpotifyHandler<'a> {
    pub client: &'a AuthCodePkceSpotify,
    pub storage: &'a storage::SpotifyStorage<'a>,
    pub xdg_dirs: &'a xdg::BaseDirectories,
}

impl<'a> SpotifyHandler<'a> {
    pub fn me(&self) {
        let current_user: PrivateUser = self
            .client
            .me()
            .expect("Unable to fetch you account information");
        println!("{:?}", current_user);
    }

    fn inbox_playlist(&self) -> SimplifiedPlaylist {
        let mut user_playlists = self.client.current_user_playlists();

        let inbox_playlist = user_playlists
            .find(|playlist| match playlist {
                Ok(playlist) => playlist.name.contains("Inbox"),
                _ => false,
            })
            .expect("No inbox playlist found")
            .unwrap();

        return inbox_playlist;
    }

    pub fn print_inbox(&self) {
        let inbox = self.inbox_playlist();
        println!(
            "Your inbox playlist is called {} and has {} tracks",
            inbox.name, inbox.tracks.total
        );
    }

    // TODO: Adjust storage path
    /// Downloads the current items in a playlist and stores them in a cache file
    pub fn cache_playlist(&self, playlist: &SimplifiedPlaylist) {
        println!(
            "Downloading playlist {} ({})",
            playlist.name,
            playlist.id.id()
        );

        let playlist_items: Vec<PlaylistItem> = self
            .client
            .playlist_items(&playlist.id, None, None)
            .filter_map(|p| p.ok())
            .collect();

        let json =
            serde_json::to_string(&playlist_items).expect("Failed to serialize playlist items");

        let playlist_filename = format!("{}.json", playlist.id.id());
        let playlist_path = self.xdg_dirs.get_cache_file(playlist_filename);

        fs::write(playlist_path, json).expect("Failed to write playlist to file");
    }

    /// Returns all playlists that are owned by the current user.
    pub fn get_user_playlists(&self) -> Vec<SimplifiedPlaylist> {
        let current_user: PrivateUser = self.client.me().unwrap();

        return self
            .client
            .current_user_playlists()
            .filter_map(|p| p.ok())
            .filter(|p| p.owner.id == current_user.id && !p.name.contains("Inbox"))
            .collect();
    }

    /// Download all playlists to cache
    pub fn playlists(&self) {
        let user_playlists = self.get_user_playlists();

        self.cache_playlist(&user_playlists[0])
    }

    // TODO: Return better data type
    /// Return the ids of all saved songs.
    pub fn get_saved_songs(&self) -> Vec<String> {
        let result: Vec<SavedTrack> = self
            .client
            .current_user_saved_tracks(None)
            .filter_map(|t| t.ok())
            .collect();

        let mut ids: Vec<String> = Vec::new();
        for track in &result {
            ids.push(track.track.id.as_ref().unwrap().id().to_string());
        }
        return ids;
    }

    pub fn get_track_ids_in_playlists(
        &self,
        playlists: &Vec<SimplifiedPlaylist>,
    ) -> HashSet<String> {
        let mut songs_in_playlists: HashSet<String> = HashSet::new();

        for playlist in playlists {
            println!("Fetching playlist {}", playlist.name);
            let playlist_items: Vec<PlaylistItem> = self
                .client
                .playlist_items(&playlist.id, None, None)
                .filter_map(|p| p.ok())
                .collect();

            for item in &playlist_items {
                let track = &item.track.as_ref().unwrap();
                match track {
                    PlayableItem::Track(track) => {
                        if let Some(id) = track.id.as_ref() {
                            songs_in_playlists.insert(id.id().to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        return songs_in_playlists;
    }

    /// Add all given tracks to a playlist.
    ///
    /// Spotify imposes a limit of 100 tracks maximum per request. If the given number of tracks is
    /// bigger, another request will be sent.
    fn playlist_add_items(&self, playlist_id: &PlaylistId, tracks: &HashSet<String>) {
        let track_ids = tracks
            .iter()
            .map(|id| TrackId::from_id(id).unwrap())
            .collect::<Vec<TrackId>>();

        track_ids.chunks(100).for_each(|chunk| {
            // TODO: Uff... find a more elegant solution for the ID conversion
            let playable = chunk
                .iter()
                .map(|id| id as &dyn PlayableId)
                .collect::<Vec<&dyn PlayableId>>();

            let result = self.client.playlist_add_items(playlist_id, playable, None);

            match result {
                Ok(_) => {
                    println!("Added {} songs to your inbox", chunk.len());
                }
                Err(e) => {
                    println!("Failed to add songs to your inbox: {}", e);
                }
            }
        });
    }

    /// Add all given tracks to a playlist.
    ///
    /// Spotify imposes a limit of 100 tracks maximum per request. If the given number of tracks is
    /// bigger, another request will be sent.
    fn playlist_remove_items(&self, playlist_id: &PlaylistId, tracks: &HashSet<String>) {
        let track_ids = tracks
            .iter()
            .map(|id| TrackId::from_id(id).unwrap())
            .collect::<Vec<TrackId>>();

        track_ids.chunks(100).for_each(|chunk| {
            // TODO: Same as above
            let playable = chunk
                .iter()
                .map(|id| id as &dyn PlayableId)
                .collect::<Vec<&dyn PlayableId>>();

            let result =
                self.client
                    .playlist_remove_all_occurrences_of_items(playlist_id, playable, None);

            match result {
                Ok(_) => {
                    println!("Removed {} songs from your inbox", chunk.len());
                }
                Err(e) => {
                    println!("Failed to remove songs to your inbox: {}", e);
                }
            }
        });
    }

    /// Flow:
    /// - Get saved songs
    /// - Get all playlists, only keep playlists created by the current user
    /// - Get all songs in user playlists
    /// - Compare songs
    /// - Get songs in inbox
    /// - Diff; add & remove songs
    pub fn sync(&self) {
        let saved_songs_vec = self.get_saved_songs();
        let saved_songs: HashSet<String> = HashSet::from_iter(saved_songs_vec.iter().cloned());

        let user_playlists = self.get_user_playlists();
        let tracks_in_playlists = self.get_track_ids_in_playlists(&user_playlists);

        let unsorted_tracks: HashSet<String> = saved_songs
            .difference(&tracks_in_playlists)
            .cloned()
            .collect();

        println!("You have {} saved songs", saved_songs.len());
        println!("You have {} unsorted songs", unsorted_tracks.len());

        let inbox_playlist = self.inbox_playlist();
        let tracks_in_inbox =
            self.get_track_ids_in_playlists(vec![inbox_playlist.clone()].as_ref());

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

        self.playlist_add_items(&inbox_playlist.id, &to_be_added);
        self.playlist_remove_items(&inbox_playlist.id, &to_be_removed)
    }
}
