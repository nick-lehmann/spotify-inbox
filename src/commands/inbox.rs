use dialoguer::{theme::ColorfulTheme, Select};
use rspotify::clients::OAuthClient;
use rspotify_model::SimplifiedPlaylist;

use crate::{config::SpotifyInboxConfig, handler::SpotifyHandler};

pub fn choose_inbox(handler: &SpotifyHandler, config: &SpotifyInboxConfig) {
    let current_inbox = config.inbox_playlist_get();

    let me = handler.me();

    let user_playlists: Vec<SimplifiedPlaylist> = handler
        .client
        .current_user_playlists()
        .filter_map(|playlist| playlist.ok())
        .filter(|playlist| playlist.owner.id == me.id)
        .collect();

    let playlist_names: Vec<&String> = user_playlists
        .iter()
        .map(|playlist| &playlist.name)
        .collect();

    let initial = match current_inbox {
        Some(playlist) => {
            let current_inbox_index = user_playlists.iter().position(|p| p.id == playlist);
            match current_inbox_index {
                Some(index) => index,
                None => {
                    println!("Current inbox playlist not found in user playlists");
                    0
                }
            }
        }
        None => {
            println!("No current inbox playlist set");
            0
        }
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your inbox")
        .default(initial)
        .items(&playlist_names)
        .interact()
        .unwrap();

    println!("Setting \"{}\" as your inbox!", playlist_names[selection]);
}
