use std::{collections::HashSet, path::PathBuf};

use rspotify::{prelude::*, scopes, AuthCodePkceSpotify, Config, Credentials, OAuth};
use rspotify_model::{PlaylistId, TrackId};

pub fn get_client(cache_path: &PathBuf) -> AuthCodePkceSpotify {
    let credentials = Credentials {
        id: "be2a290c5f2c4208af53c58952fc7af5".to_string(),
        secret: Some("".to_string()),
    };

    let oauth = OAuth {
        scopes: scopes![
            "ugc-image-upload",
            "user-read-playback-state",
            "user-modify-playback-state",
            "user-read-currently-playing",
            "streaming",
            "app-remote-control",
            "user-read-email",
            "user-read-private",
            "playlist-read-collaborative",
            "playlist-modify-public",
            "playlist-read-private",
            "playlist-modify-private",
            "user-library-modify",
            "user-library-read",
            "user-top-read",
            "user-read-playback-position",
            "user-read-recently-played",
            "user-follow-read",
            "user-follow-modify"
        ],
        redirect_uri: "http://localhost:8888/callback".to_string(),
        ..Default::default()
    };

    let config = Config {
        token_refreshing: true,
        token_cached: true,
        cache_path: cache_path.to_path_buf(),
        ..Default::default()
    };

    let mut spotify = AuthCodePkceSpotify::with_config(credentials, oauth, config);
    let url = spotify.get_authorize_url(None).unwrap();
    spotify.prompt_for_token(&url).unwrap();

    return spotify;
}

pub trait SpotifyHelper {
    fn playlist_add_all_items(&self, playlist_id: &PlaylistId, tracks: &HashSet<String>);
    fn playlist_remove_all_items(&self, playlist_id: &PlaylistId, tracks: &HashSet<String>);
}

impl SpotifyHelper for AuthCodePkceSpotify {
    /// Add all given tracks to a playlist.
    ///
    /// Spotify imposes a limit of 100 tracks maximum per request. If the given number of tracks is
    /// bigger, another request will be sent.
    // TODO: Create PR to rspotify.
    fn playlist_add_all_items(&self, playlist_id: &PlaylistId, tracks: &HashSet<String>) {
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

            let result = self.playlist_add_items(playlist_id, playable, None);

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
    // TODO: Create PR to rspotify.
    fn playlist_remove_all_items(&self, playlist_id: &PlaylistId, tracks: &HashSet<String>) {
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

            let result = self.playlist_remove_all_occurrences_of_items(playlist_id, playable, None);

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
}
