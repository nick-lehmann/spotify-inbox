use indicatif::ProgressBar;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    AuthCodePkceSpotify,
};
use rspotify_model::{
    Id, Page, PlayableItem, PlaylistItem, PrivateUser, SavedTrack, SimplifiedPlaylist,
};
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

    /// Return the ids of all saved songs.
    // pub fn get_saved_songs(&self) -> Vec<String> {
    #[allow(unreachable_code)]
    pub fn saved_songs(&self) -> Vec<SavedTrack> {
        return self.saved_songs_download();

        // TODO: Skip if None received
        let mut cached_saved_songs = match self.storage.get_saved_songs() {
            Some(saved_songs) => saved_songs,
            None => return self.saved_songs_download(),
        };

        let total_cached = cached_saved_songs.len() as u32;

        cached_saved_songs.sort_unstable_by_key(|s| s.added_at);
        cached_saved_songs.reverse();

        let page_size = 50;
        let saved_songs_page: Page<SavedTrack> = self
            .client
            .current_user_saved_tracks_manual(None, Some(page_size), Some(0))
            .unwrap();
        let total = saved_songs_page.total;
        let mut saved_songs = saved_songs_page.items;

        saved_songs.sort_unstable_by_key(|s| s.added_at);
        saved_songs.reverse();

        let latest_addition = saved_songs[0].added_at;
        let latest_addition_cached = cached_saved_songs[0].added_at;

        if total == total_cached {
            if latest_addition == latest_addition_cached {
                // println!("No new songs saved");
                cached_saved_songs
            } else {
                // println!("We have the right amount of songs, but the latest addition is different. Refetch just to be sure");
                self.saved_songs_download()
            }
        } else if latest_addition > latest_addition_cached {
            // There are new songs that we have not cached yet, probably.
            // println!("We have more songs than we have cached. Refetch just to be sure");

            // Check if the downloaded songs are already enough
            let new_songs: Vec<SavedTrack> = saved_songs
                .into_iter()
                .filter(|song| song.added_at > latest_addition_cached)
                .collect();
            if new_songs.len() == page_size as usize {
                // Just download everything
                return self.saved_songs_download();
            }

            let mut all_saved_songs: Vec<SavedTrack> = Vec::new();
            all_saved_songs.extend(new_songs);
            all_saved_songs.extend(cached_saved_songs);

            all_saved_songs
        } else {
            // There were songs removed from the front that are still cached, probably.
            println!("We have less songs than we have cached. Refetch just to be sure");
            self.saved_songs_download()
        }
    }

    fn saved_songs_download(&self) -> Vec<SavedTrack> {
        self.client
            .current_user_saved_tracks(None)
            .filter_map(|t| t.ok())
            .collect()
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
