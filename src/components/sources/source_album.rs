use rspotify::model::*;
use rspotify::prelude::*;
use rspotify::AuthCodeSpotify as Client;

use serde::{Deserialize, Serialize};

use crate::components::{Executable, Result, TrackList};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AlbumArgs {
    pub id: String,
}

pub struct Album;

impl Executable for Album {
    type Args = AlbumArgs;

    // Fetch the list of tracks in the album, then
    // request the FullTrack object
    fn execute(client: &Client, args: Self::Args, _: Vec<TrackList>) -> Result<TrackList> {
        let mut ids = Vec::new(); // Temp track id vector
        for t in client.album_track(
            AlbumId::from_id_or_uri(&args.id).unwrap(),
            Some(Market::FromToken),
        ) {
            ids.push(t.unwrap().id.unwrap())
        }
        client.tracks(ids, None).map_err(|e| e.into())
    }
}
