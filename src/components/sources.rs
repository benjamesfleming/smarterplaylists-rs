///! Sources take user-defined arguments and return TrackLists
use rspotify::model::Market;
use rspotify::prelude::*;
use rspotify::AuthCodeSpotify as Client;

use super::TrackList;

pub trait Source {
    fn source(&self, client: &Client) -> Result<TrackList, rspotify::ClientError>;
}

//

pub type Album = rspotify::model::FullAlbum;

impl Source for Album {
    // Fetch the list of tracks in the album, then
    // request the FullTrack object
    fn source(&self, client: &Client) -> Result<TrackList, rspotify::ClientError> {
        let mut ids = Vec::new(); // Temp track id vector
        for t in client.album_track(self.id.to_owned()) {
            ids.push(t.unwrap().id.unwrap())
        }
        client.tracks(ids, None)
    }
}

//

pub type Artist = rspotify::model::FullArtist;

impl Source for Artist {
    // Fetch top tracks for a given artist
    // Note: This selects the artists top tracks, not all of them
    fn source(&self, client: &Client) -> Result<TrackList, rspotify::ClientError> {
        client.artist_top_tracks(self.id.to_owned(), Market::FromToken)
    }
}

//

// pub struct SpotifyPlaylist;
// pub struct PrivatePlaylist;
