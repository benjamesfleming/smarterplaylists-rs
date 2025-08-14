use rspotify::model::*;
use rspotify::prelude::*;
use rspotify::AuthCodeSpotify as Client;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::components::{Executable, Result, TrackList};

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ArtistTopTracksArgs {
    pub id: String,
}

pub struct ArtistTopTracks;

impl Executable for ArtistTopTracks {
    type Args = ArtistTopTracksArgs;

    // Fetch top tracks for a given artist
    // Note: This selects the artists top tracks, not all of them
    fn execute(client: &Client, args: Self::Args, _: Vec<TrackList>) -> Result<TrackList> {
        client
            .artist_top_tracks(
                ArtistId::from_id_or_uri(&args.id).unwrap(),
                Some(Market::FromToken),
            )
            .map_err(|e| e.into())
    }
}
