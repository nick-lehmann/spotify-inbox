use crate::{cli::run, handler::SpotifyHandler, spotify::get_client};

mod cli;
mod handler;
mod spotify;
mod storage;

extern crate xdg;

const APP_NAME: &str = "spotify-inbox";

fn main() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME).unwrap();
    let storage = storage::SpotifyStorage::new(&xdg_dirs);

    let client = get_client(&storage.path_helper.get_cache_path());

    let handler = SpotifyHandler {
        client: &client,
        storage: &storage,
        xdg_dirs: &xdg_dirs,
    };

    run(&handler);
}
