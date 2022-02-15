use indicatif::ProgressBar;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    AuthCodePkceSpotify,
};
use rspotify_model::{Id, PlayableItem, PlaylistItem, PrivateUser, SimplifiedPlaylist};
use std::collections::HashSet;

use crate::storage;

pub struct SpotifyHandler<'a> {
    pub client: &'a AuthCodePkceSpotify,
    pub storage: &'a storage::SpotifyStorage<'a>,
}

impl<'a> SpotifyHandler<'a> {
    pub fn me(&self) -> PrivateUser {
        self.client
            .me()
            .expect("Unable to fetch you account information")
    }

    pub fn playlist_find_inbox(&self) -> SimplifiedPlaylist {
        let mut user_playlists = self.client.current_user_playlists();

        user_playlists
            .find(|playlist| match playlist {
                Ok(playlist) => playlist.name.contains("Inbox"),
                _ => false,
            })
            .expect("No inbox playlist found")
            .unwrap()
    }

    pub fn playlist_get(&self, playlist: &SimplifiedPlaylist) -> storage::CompletePlaylist {
        let cached_playlist = self.storage.get_playlist(&playlist.id);

        match cached_playlist {
            Some(cached_playlist) => {
                if cached_playlist.snapshot_id == playlist.snapshot_id {
                    // println!("Using cached playlist");
                    return cached_playlist;
                } else {
                    // println!("Cached playlist is out of date");
                }
            }
            None => {
                // println!("No cached playlist found");
            }
        }

        let playlist_tracks: Vec<PlaylistItem> = self
            .client
            .playlist_items(&playlist.id, None, None)
            .filter_map(|p| p.ok())
            .collect();

        let full_playlist = storage::CompletePlaylist {
            tracks: playlist_tracks,
            collaborative: playlist.collaborative,
            external_urls: playlist.external_urls.to_owned(),
            href: playlist.href.to_owned(),
            id: playlist.id.to_owned(),
            images: playlist.images.to_owned(),
            name: playlist.name.to_owned(),
            owner: playlist.owner.to_owned(),
            public: playlist.public.to_owned(),
            snapshot_id: playlist.snapshot_id.to_owned(),
        };

        self.storage.write_playlist(&full_playlist);

        full_playlist
    }

    /// Returns all playlists that are owned by the current user.
    pub fn playlist_get_owned_by_user(&self) -> Vec<SimplifiedPlaylist> {
        let current_user: PrivateUser = self.client.me().unwrap();

        return self
            .client
            .current_user_playlists()
            .filter_map(|p| p.ok())
            .filter(|p| p.owner.id == current_user.id && !p.name.contains("Inbox"))
            .collect();
    }

    pub fn get_track_ids_in_playlists(
        &self,
        playlists: &Vec<SimplifiedPlaylist>,
    ) -> HashSet<String> {
        let mut songs_in_playlists: HashSet<String> = HashSet::new();

        let pb = ProgressBar::new(playlists.len() as u64);

        for playlist in playlists {
            let playlist = self.playlist_get(playlist);

            pb.inc(1);

            for item in &playlist.tracks {
                let track = item.track.as_ref().unwrap();
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

        pb.finish_with_message("done 👍🏻");

        songs_in_playlists
    }
}
