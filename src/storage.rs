use std::path::PathBuf;

use rspotify_model::{Id, PlaylistId, SimplifiedPlaylist};

use crate::APP_NAME;

pub struct SpotifyStorage<'a> {
    pub xdg_dirs: &'a xdg::BaseDirectories,
}

impl<'a> SpotifyStorage<'a> {
    fn get_cache_path(&self) -> PathBuf {
        let config_name = format!("{}.json", APP_NAME);
        let config_path = self
            .xdg_dirs
            .find_cache_file(config_name)
            .expect("Unable to find cache file");
        config_path
    }

    pub fn get_playlist(&self, id: &PlaylistId) -> Option<SimplifiedPlaylist> {
        let cache_path = self
            .xdg_dirs
            .get_cache_file(format!("playlists/{}.json", id.id()));

        println!("Checking if {} exists", cache_path.to_string_lossy());

        let playlist_string = match std::fs::read_to_string(&cache_path) {
            Ok(playlist_string) => playlist_string,
            Err(_) => return None,
        };

        let playlist: SimplifiedPlaylist = serde_json::from_str(&playlist_string).unwrap();

        Some(playlist)
    }
}
