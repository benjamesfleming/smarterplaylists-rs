//! DeduplicateArtist filter removes tracks with duplicate primary artists, keeping only the first occurrence
use rspotify::prelude::Id;
use rspotify::AuthCodeSpotify as Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::components::{Executable, Result, TrackList};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DeduplicateArtistArgs {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DeduplicateArtist;

impl Executable for DeduplicateArtist {
    type Args = DeduplicateArtistArgs;

    fn execute(_: &Client, _args: Self::Args, prev: Vec<TrackList>) -> Result<TrackList> {
        if prev.is_empty() {
            return Ok(Vec::new());
        }

        let tracks = prev.first().unwrap();
        let mut seen_artists = HashSet::new();
        let mut result = Vec::new();

        for track in tracks {
            // Only check the primary artist (first in the list)
            if let Some(primary_artist) = track.artists.first() {
                let artist_id = primary_artist
                    .id
                    .as_ref()
                    .map(|id| id.id())
                    .unwrap_or_else(|| primary_artist.name.as_str());

                // If this is the first time we've seen this artist, add the track
                if seen_artists.insert(artist_id.to_string()) {
                    result.push(track.clone());
                }
            } else {
                // Track has no artists, keep it anyway (edge case)
                result.push(track.clone());
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use rspotify::model::{ArtistId, FullTrack, SimplifiedArtist, TrackId};

    // Helper function to create a test track with a given ID and artist ID
    fn create_test_track(id: &str, artist_ids: Vec<&str>) -> FullTrack {
        let track_id = TrackId::from_id(id.to_owned()).ok();

        let artists = artist_ids
            .into_iter()
            .map(|artist_id| SimplifiedArtist {
                id: ArtistId::from_id(artist_id.to_owned()).ok(),
                name: format!("Artist {}", artist_id),
                external_urls: Default::default(),
                href: None,
            })
            .collect();

        FullTrack {
            id: track_id,
            artists,
            name: format!("Track {}", id),
            album: Default::default(),
            available_markets: vec![],
            disc_number: 1,
            duration: TimeDelta::seconds(180), // 3 minutes
            explicit: false,
            external_ids: Default::default(),
            external_urls: Default::default(),
            href: None,
            is_local: false,
            is_playable: None,
            linked_from: None,
            popularity: 0,
            preview_url: None,
            restrictions: None,
            track_number: 1,
        }
    }

    #[test]
    fn test_dedup_primary_artist() {
        // Create a test track list with some primary artist duplicates
        let mut tracklist = Vec::new();

        // Track 1 - Artist A
        tracklist.push(create_test_track("track1", vec!["artistA"]));

        // Track 2 - Artist B
        tracklist.push(create_test_track("track2", vec!["artistB"]));

        // Track 3 - Artist A again (should be removed)
        tracklist.push(create_test_track("track3", vec!["artistA"]));

        // Track 4 - Artist C
        tracklist.push(create_test_track("track4", vec!["artistC"]));

        // Track 5 - Artist B again (should be removed)
        tracklist.push(create_test_track("track5", vec!["artistB"]));

        // Execute the component
        let args = DeduplicateArtistArgs {};

        let result =
            DeduplicateArtist::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();

        // Verify the result
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, tracklist[0].id); // Track 1 - Artist A
        assert_eq!(result[1].id, tracklist[1].id); // Track 2 - Artist B
        assert_eq!(result[2].id, tracklist[3].id); // Track 4 - Artist C
    }

    #[test]
    fn test_dedup_with_collaborations() {
        // Create a test track list with collaborations (multiple artists on one track)
        let mut tracklist = Vec::new();

        // Track 1 - Primary Artist A (with secondary B)
        tracklist.push(create_test_track("track1", vec!["artistA", "artistB"]));

        // Track 2 - Primary Artist C (with secondary A)
        // This track should be kept because C is the primary artist, even though A appears as secondary
        tracklist.push(create_test_track("track2", vec!["artistC", "artistA"]));

        // Track 3 - Primary Artist A again (with secondary D)
        // This track should be removed because A is primary artist and has been seen before
        tracklist.push(create_test_track("track3", vec!["artistA", "artistD"]));

        // Track 4 - Primary Artist B
        // This track should be kept because B is primary here, but was secondary in Track 1
        tracklist.push(create_test_track("track4", vec!["artistB"]));

        // Execute the component
        let args = DeduplicateArtistArgs {};

        let result =
            DeduplicateArtist::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();

        // Verify the result
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, tracklist[0].id); // Track 1 - Primary Artist A
        assert_eq!(result[1].id, tracklist[1].id); // Track 2 - Primary Artist C
        assert_eq!(result[2].id, tracklist[3].id); // Track 4 - Primary Artist B
    }

    #[test]
    fn test_empty_tracklist() {
        // Execute the component with an empty track list
        let args = DeduplicateArtistArgs {};

        let result =
            DeduplicateArtist::execute(&Client::default(), args, vec![Vec::new()]).unwrap();

        // Verify the result is empty
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_no_artists() {
        // Create a track with no artists (edge case)
        let mut tracklist = Vec::new();
        tracklist.push(create_test_track("track1", vec![]));

        // Execute the component
        let args = DeduplicateArtistArgs {};

        let result =
            DeduplicateArtist::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();

        // Verify the track with no artists is kept
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, tracklist[0].id);
    }
}
