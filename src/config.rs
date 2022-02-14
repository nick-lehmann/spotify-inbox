use std::{fs::File, io::Write, path::PathBuf};

use rspotify_model::{Id, PlaylistId};
use serde::{Deserialize, Serialize};

use crate::APP_NAME;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    inbox: Option<PlaylistId>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inbox: Default::default(),
        }
    }
}

pub struct SpotifyInboxConfig {
    pub xdg_dirs: xdg::BaseDirectories,
}

impl SpotifyInboxConfig {
    pub fn new(xdg_dirs: xdg::BaseDirectories) -> Self {
        SpotifyInboxConfig { xdg_dirs }
    }

    pub fn get_saved_songs_cache_path(&self) -> PathBuf {
        self.xdg_dirs.get_cache_file("saved-songs.json")
    }

    pub fn get_playlist_path(&self, id: &PlaylistId) -> PathBuf {
        return self
            .xdg_dirs
            .get_cache_file(format!("playlists/{}.json", id.id()));
    }

    /**
     * Return the path to the config file.
     *
     * Unix: ~/.config/spotify-inbox/spotify-inbox.json
     */
    fn get_config_path(&self) -> PathBuf {
        self.xdg_dirs.get_config_file(format!("{}.json", APP_NAME))
    }

    /**
     * Reads the config file from disk.
     */
    fn read_config(&self) -> Config {
        let config_path = self.get_config_path();

        let config_string = match std::fs::read_to_string(&config_path) {
            Ok(config_string) => config_string,
            Err(_) => return Config::default(),
        };

        let config: Config = serde_json::from_str(&config_string).unwrap();

        config
    }

    /**
     * Write the updated config to disk.
     */
    fn write_config(&self, config: &Config) {
        let config_path = self.get_config_path();

        let config_string = serde_json::to_string_pretty(&config).unwrap();

        let mut file = File::create(&config_path).unwrap();
        file.write(config_string.as_bytes()).unwrap();
    }

    pub fn inbox_playlist_get(&self) -> Option<PlaylistId> {
        let config = self.read_config();

        config.inbox
    }

    pub fn inbox_playlist_set(&self, inbox: &PlaylistId) {
        let config = self.read_config();

        let mut new_config = config.clone();
        new_config.inbox = Some(inbox.clone());

        self.write_config(&new_config);
    }

    #[allow(dead_code)]
    pub fn get_cache_path(&self) -> PathBuf {
        let config_name = format!("{}.json", APP_NAME);

        self.xdg_dirs.get_cache_file(config_name)
    }
}
