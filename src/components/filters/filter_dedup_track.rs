//! DeduplicateTrack filter removes duplicate tracks based on their Spotify track IDs
use rspotify::AuthCodeSpotify as Client;
use rspotify::prelude::Id;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::components::{Executable, Result, TrackList};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DeduplicateTrackArgs {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DeduplicateTrack;

impl Executable for DeduplicateTrack {
    type Args = DeduplicateTrackArgs;

    fn execute(_: &Client, _args: Self::Args, prev: Vec<TrackList>) -> Result<TrackList> {
        if prev.is_empty() {
            return Ok(Vec::new());
        }

        let tracks = prev.first().unwrap();
        let mut seen_track_ids = HashSet::new();
        let mut result = Vec::new();

        for track in tracks {
            // Get track ID or use a combination of name and artist as fallback
            let track_id = track.id.as_ref()
                .map(|id| id.id().to_string())
                .unwrap_or_else(|| {
                    // For tracks without ID (local files, etc.), use a combination of name and primary artist
                    let artist_name = track.artists.first()
                        .map(|artist| artist.name.as_str())
                        .unwrap_or("");
                    format!("{}:{}", track.name, artist_name)
                });
            
            // If this is the first time we've seen this track, add it
            if seen_track_ids.insert(track_id) {
                result.push(track.clone());
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspotify::model::{FullTrack, SimplifiedArtist, TrackId, ArtistId};
    use time::Duration;
    
    // Helper function to create a test track with a given ID and artist ID
    fn create_test_track(id: &str, artist_ids: Vec<&str>) -> FullTrack {
        let track_id = TrackId::from_id(id.to_owned()).ok();
        
        let artists = artist_ids.into_iter().map(|artist_id| {
            SimplifiedArtist {
                id: ArtistId::from_id(artist_id.to_owned()).ok(),
                name: format!("Artist {}", artist_id),
                external_urls: Default::default(),
                href: None,
            }
        }).collect();
        
        FullTrack {
            id: track_id,
            artists,
            name: format!("Track {}", id),
            album: Default::default(),
            available_markets: vec![],
            disc_number: 1,
            duration: Duration::seconds(180), // 3 minutes
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
    fn test_dedup_tracks() {
        // Create a test track list with some duplicate tracks
        let mut tracklist = Vec::new();
        
        // Unique track 1
        tracklist.push(create_test_track("track1", vec!["artistA"]));
        
        // Unique track 2
        tracklist.push(create_test_track("track2", vec!["artistB"]));
        
        // Duplicate of track 1 (should be removed)
        tracklist.push(create_test_track("track1", vec!["artistA"]));
        
        // Unique track 3
        tracklist.push(create_test_track("track3", vec!["artistC"]));
        
        // Duplicate of track 2 (should be removed)
        tracklist.push(create_test_track("track2", vec!["artistB"]));
        
        // Execute the component
        let args = DeduplicateTrackArgs {};
        
        let result = DeduplicateTrack::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();
        
        // Verify the result
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, tracklist[0].id); // Track 1
        assert_eq!(result[1].id, tracklist[1].id); // Track 2
        assert_eq!(result[2].id, tracklist[3].id); // Track 3
    }
    
    #[test]
    fn test_tracks_without_ids() {
        // Create test tracks without IDs to test the fallback mechanism
        let mut tracklist = Vec::new();
        
        // Create two tracks with same name but different artists (should both be kept)
        let mut track1 = create_test_track("track1", vec!["artistA"]);
        let mut track2 = create_test_track("track1", vec!["artistB"]);
        
        // Simulate local tracks by removing the IDs
        track1.id = None;
        track2.id = None;
        
        // Create a duplicate of track1 (should be removed)
        let mut track3 = create_test_track("track1", vec!["artistA"]);
        track3.id = None;
        
        tracklist.push(track1);
        tracklist.push(track2);
        tracklist.push(track3);
        
        // Execute the component
        let args = DeduplicateTrackArgs {};
        
        let result = DeduplicateTrack::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();
        
        // Verify the result
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Track track1");
        assert_eq!(result[0].artists[0].name, "Artist artistA");
        assert_eq!(result[1].name, "Track track1");
        assert_eq!(result[1].artists[0].name, "Artist artistB");
    }
    
    #[test]
    fn test_empty_tracklist() {
        // Execute the component with an empty track list
        let args = DeduplicateTrackArgs {};
        
        let result = DeduplicateTrack::execute(&Client::default(), args, vec![Vec::new()]).unwrap();
        
        // Verify the result is empty
        assert_eq!(result.len(), 0);
    }
    
    #[test]
    fn test_tracks_without_artists() {
        // Create a track with no artists
        let mut tracklist = Vec::new();
        let track = create_test_track("track1", vec![]);
        tracklist.push(track.clone());
        
        // Add a duplicate (should be removed)
        tracklist.push(track);
        
        // Execute the component
        let args = DeduplicateTrackArgs {};
        
        let result = DeduplicateTrack::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();
        
        // Verify only one track remains
        assert_eq!(result.len(), 1);
    }
}