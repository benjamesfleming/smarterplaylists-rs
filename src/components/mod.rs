/// The TrackList is a collection of FullTracks. It is used
/// as a return type for source components
pub type TrackList = Vec<rspotify::model::FullTrack>;

pub mod combiners;
pub mod conditinals;
pub mod filters;
pub mod sources;
