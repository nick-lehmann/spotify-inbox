use crate::{cli::run, handler::SpotifyHandler, spotify::get_client};

mod cli;
mod handler;
mod spotify;
mod storage;

extern crate xdg;

const APP_NAME: &str = "spotify-inbox";

fn main() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME).unwrap();
    let config_name = format!("{}.json", APP_NAME);

    let cache_dir = xdg_dirs
        .create_cache_directory(APP_NAME)
        .expect("Unable to create cache directory");
    println!("Cache dir: {:?}", cache_dir);

    let storage = storage::SpotifyStorage {
        xdg_dirs: &xdg_dirs,
    };

    // let token = spotify.token.as_ref().lock().unwrap();
    // println!("Access token: {}", token.as_ref().unwrap().access_token);
    // println!("Refresh token: {:?}", token.as_ref().unwrap().refresh_token);

    let client = get_client(&cache_dir.join(config_name));

    let handler = SpotifyHandler {
        client: &client,
        storage: &storage,
        xdg_dirs: &xdg_dirs,
    };

    run(&handler);
}
