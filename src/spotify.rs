use std::path::PathBuf;

use rspotify::{prelude::*, scopes, AuthCodePkceSpotify, Config, Credentials, OAuth};

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
