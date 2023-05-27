/// The TrackList is a collection of FullTracks. It is used
/// as a return type for source components
pub type TrackList = Vec<rspotify::model::FullTrack>;

pub mod combiners;
pub mod conditinals;
pub mod filters;
pub mod sources;

use rspotify::AuthCodeSpotify as Client;
use serde::Deserialize;

use crate::error::PublicError;

use self::sources::*;

pub trait Component<T> {
    fn execute(
        client: &Client,
        args: T,
        prev: Vec<TrackList>,
    ) -> Result<TrackList, rspotify::ClientError>;
}

// --

macro_rules! components {
    ( $(($a:literal, $b:ty, $c:ty)),* ) => {
        // Execute a component by name -
        // n.b This function takes a generic args value and will fail if it can't be deserilized into the correct type
        pub fn execute(id: &str, client: &Client, args: serde_json::Value, prev: Vec<TrackList>) -> Result<TrackList, PublicError> {
            match id {
                $(
                    $a => <$b>::execute(client, <$c as Deserialize>::deserialize(args)?, prev).map_err(|_| "".into()),
                )*
                _ => Err(format!("invalid component type '{}' provided", id).into()),
            }
        }
    };
}

#[rustfmt::skip::macros(components)]
components![
    ("source:artist_top_tracks", ArtistTopTracks, ArtistTopTracksArgs),
    ("source:album", Album, AlbumArgs)
];
