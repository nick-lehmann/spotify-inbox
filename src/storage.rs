use std::{fs::File, io::Write};

use crate::config::SpotifyInboxConfig;
use rspotify_model::{Image, PlaylistId, PlaylistItem, PublicUser};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CompletePlaylist {
    pub collaborative: bool,
    pub external_urls: HashMap<String, String>,
    pub href: String,
    pub id: PlaylistId,
    pub images: Vec<Image>,
    pub name: String,
    pub owner: PublicUser,
    pub public: Option<bool>,
    pub snapshot_id: String,
    pub tracks: Vec<PlaylistItem>,
}

pub struct SpotifyStorage<'a> {
    pub config: &'a SpotifyInboxConfig,
}

impl<'a> SpotifyStorage<'a> {
    pub fn new(config: &'a SpotifyInboxConfig) -> Self {
        SpotifyStorage { config: config }
    }

    pub fn get_playlist(&self, id: &PlaylistId) -> Option<CompletePlaylist> {
        let cache_path = self.config.get_playlist_path(id);

        let playlist_string = match std::fs::read_to_string(&cache_path) {
            Ok(playlist_string) => playlist_string,
            Err(_) => return None,
        };

        let playlist: CompletePlaylist = serde_json::from_str(&playlist_string).unwrap();

        Some(playlist)
    }

    pub fn write_playlist(&self, playlist: &CompletePlaylist) {
        let cache_path = self.config.get_playlist_path(&playlist.id);

        fs::create_dir_all(cache_path.parent().unwrap()).unwrap();

        let json_string =
            serde_json::to_string_pretty(&playlist).expect("Unable to serialize playlist");

        let mut file = File::create(cache_path).expect("Unable to create cache file for playlist");
        file.write_all(json_string.as_bytes())
            .expect("Unable to write to playlist to cache");
    }
}
