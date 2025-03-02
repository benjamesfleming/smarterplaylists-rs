//! Filters do work on one source TrackList, returning it after filtering
pub mod filter_dedup_artist;
pub mod filter_dedup_track;
pub mod filter_take;

pub use filter_dedup_artist::*;
pub use filter_dedup_track::*;
pub use filter_take::*;
