///! Filters do work on one source TrackList, returning it after filtering
use rspotify::AuthCodeSpotify as Client;
use serde::{Deserialize, Serialize};

use super::Result;
use super::*;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TakeArgs {
    pub limit: u32,
    pub from: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Take;

impl Executable for Take {
    type Args = TakeArgs;

    fn execute(_: &Client, args: Self::Args, prev: Vec<TrackList>) -> Result<TrackList> {
        let tracks = prev.first().unwrap().iter();
        if args.from.eq("end") {
            // Reverse the TrackList and take the last X tracks
            Ok(tracks.rev().take(args.limit as usize).cloned().collect())
        } else {
            // Take the first X tracks
            Ok(tracks.take(args.limit as usize).cloned().collect())
        }
    }
}

// pub struct TrackDedupFilter;
// pub struct ArtistDedupFilter;
