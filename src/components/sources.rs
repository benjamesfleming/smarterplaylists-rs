///! Sources take user-defined arguments and return TrackLists
use rspotify::model::*;
use rspotify::prelude::*;
use rspotify::AuthCodeSpotify as Client;

use serde::Deserialize;

use super::Result;
use super::*;

#[derive(Deserialize)]
pub struct AlbumArgs {
    pub id: String,
}

pub struct Album;

impl Component<AlbumArgs> for Album {
    // Fetch the list of tracks in the album, then
    // request the FullTrack object
    fn execute(client: &Client, args: AlbumArgs, _: Vec<TrackList>) -> Result<TrackList> {
        let mut ids = Vec::new(); // Temp track id vector
        for t in client.album_track(AlbumId::from_id_or_uri(&args.id).unwrap()) {
            ids.push(t.unwrap().id.unwrap())
        }
        client.tracks(ids, None).map_err(|e| e.into())
    }
}

// --

#[derive(Deserialize)]
pub struct ArtistTopTracksArgs {
    pub id: String,
}

pub struct ArtistTopTracks;

impl Component<ArtistTopTracksArgs> for ArtistTopTracks {
    // Fetch top tracks for a given artist
    // Note: This selects the artists top tracks, not all of them
    fn execute(client: &Client, args: ArtistTopTracksArgs, _: Vec<TrackList>) -> Result<TrackList> {
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
