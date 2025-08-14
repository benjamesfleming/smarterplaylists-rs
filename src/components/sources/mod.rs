///! Sources take user-defined arguments and return TrackLists
pub mod source_album;
pub mod source_artist_top_tracks;
pub mod source_user_liked_tracks;

pub use source_album::*;
pub use source_artist_top_tracks::*;
pub use source_user_liked_tracks::*;
