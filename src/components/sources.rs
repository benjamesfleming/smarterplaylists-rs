///! Sources take user-defined arguments and return TrackLists
use rspotify::model::*;
use rspotify::prelude::*;
use rspotify::AuthCodeSpotify as Client;

use serde::{Deserialize, Serialize};

use super::Result;
use super::*;

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
        for t in client.album_track(AlbumId::from_id_or_uri(&args.id).unwrap()) {
            ids.push(t.unwrap().id.unwrap())
        }
        client.tracks(ids, None).map_err(|e| e.into())
    }
}

// --

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ArtistTopTracksArgs {
    pub id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ArtistTopTracks;

impl Executable for ArtistTopTracks {
    type Args = ArtistTopTracksArgs;

    // Fetch top tracks for a given artist
    // Note: This selects the artists top tracks, not all of them
    fn execute(client: &Client, args: Self::Args, _: Vec<TrackList>) -> Result<TrackList> {
        client
            .artist_top_tracks(
                ArtistId::from_id_or_uri(&args.id).unwrap(),
                Market::FromToken,
            )
            .map_err(|e| e.into())
    }
}

//

// pub struct SpotifyPlaylist;
// pub struct PrivatePlaylist;
