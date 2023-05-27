///! Sources take user-defined arguments and return TrackLists
use rspotify::model::*;
use rspotify::prelude::*;
use rspotify::AuthCodeSpotify as Client;

use serde::Deserialize;

use super::*;

#[derive(Deserialize)]
pub struct AlbumArgs {
    pub id: String,
}

pub struct Album;

impl Component<AlbumArgs> for Album {
    // Fetch the list of tracks in the album, then
    // request the FullTrack object
    fn execute(
        client: &Client,
        args: AlbumArgs,
        _: Vec<TrackList>,
    ) -> Result<TrackList, rspotify::ClientError> {
        let mut ids = Vec::new(); // Temp track id vector
        for t in client.album_track(AlbumId::from_id_or_uri(&args.id).unwrap()) {
            ids.push(t.unwrap().id.unwrap())
        }
        client.tracks(ids, None)
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
    fn execute(
        client: &Client,
        args: ArtistTopTracksArgs,
        _: Vec<TrackList>,
    ) -> Result<TrackList, rspotify::ClientError> {
        client.artist_top_tracks(
            ArtistId::from_id_or_uri(&args.id).unwrap(),
            Market::FromToken,
        )
    }
}

//

// pub struct SpotifyPlaylist;
// pub struct PrivatePlaylist;
