use rspotify::model::*;
use rspotify::prelude::*;
use rspotify::AuthCodeSpotify as Client;

use serde::{Deserialize, Serialize};

use crate::components::{Executable, Result, TrackList};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserLikedTracksArgs {
    pub limit: u32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserLikedTracks;

impl Executable for UserLikedTracks {
    type Args = UserLikedTracksArgs;

    // Fetch users liked songs
    // Note: Limited by most recent [1-999]
    fn execute(client: &Client, args: Self::Args, prev: Vec<TrackList>) -> Result<TrackList> {
        let mut tracks = TrackList::new();
        let mut offset = 0;
        loop {
            let page = client.current_user_saved_tracks_manual(
                Some(Market::FromToken),
                Some(50),
                Some(offset),
            )?;
            if offset >= 949 || page.items.is_empty() {
                break;
            }
            offset += page.items.len() as u32;
            tracks.extend(page.items.iter().map(|st| st.track.clone()));
        }
        Ok(tracks)
    }
}
