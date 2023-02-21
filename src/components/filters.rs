///! Filters do work on one source TrackList, returning it after filtering
use super::TrackList;

pub trait Filter {
    fn apply(&self, source: TrackList) -> TrackList;
}

pub struct TrackDedupFilter;
pub struct ArtistDedupFilter;
