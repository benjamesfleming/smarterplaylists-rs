//! Take filter limits the number of tracks from beginning or end
use rspotify::AuthCodeSpotify as Client;
use serde::{Deserialize, Serialize};

use crate::components::{Executable, Result, TrackList};

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use rspotify::model::{FullTrack, SimplifiedArtist, TrackId};

    // Helper function to create a test track with a given ID
    fn create_test_track(id: &str) -> FullTrack {
        // First get the structure correct by checking a newer version of rspotify docs
        let track_id = TrackId::from_id(id.to_owned());
        let artist = SimplifiedArtist {
            id: None,
            name: format!("Artist {}", id),
            external_urls: Default::default(),
            href: None,
        };

        FullTrack {
            id: track_id.ok(),
            artists: vec![artist],
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

    // Helper function to create a test track list with sequential IDs
    fn create_test_tracklist(count: usize) -> TrackList {
        (1..=count)
            .map(|i| create_test_track(&format!("track{}", i)))
            .collect()
    }

    #[test]
    fn test_take_beginning() {
        // Create a test track list with 10 tracks
        let tracklist = create_test_tracklist(10);

        // Create Take component with parameters to take 3 from beginning
        let args = TakeArgs {
            limit: 3,
            from: "beginning".to_string(),
        };

        // Execute the component
        let result = Take::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();

        // Verify the result
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, tracklist[0].id);
        assert_eq!(result[1].id, tracklist[1].id);
        assert_eq!(result[2].id, tracklist[2].id);
    }

    #[test]
    fn test_take_end() {
        // Create a test track list with 10 tracks
        let tracklist = create_test_tracklist(10);

        // Create Take component with parameters to take 3 from end
        let args = TakeArgs {
            limit: 3,
            from: "end".to_string(),
        };

        // Execute the component
        let result = Take::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();

        // Verify the result
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, tracklist[9].id);
        assert_eq!(result[1].id, tracklist[8].id);
        assert_eq!(result[2].id, tracklist[7].id);
    }

    #[test]
    fn test_take_limit_exceeds_length() {
        // Create a test track list with 5 tracks
        let tracklist = create_test_tracklist(5);

        // Create Take component with parameters to take 10 from beginning
        let args = TakeArgs {
            limit: 10,
            from: "beginning".to_string(),
        };

        // Execute the component
        let result = Take::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();

        // Verify that we get all 5 tracks (not more)
        assert_eq!(result.len(), 5);
        for i in 0..5 {
            assert_eq!(result[i].id, tracklist[i].id);
        }
    }

    #[test]
    fn test_take_zero_limit() {
        // Create a test track list with 5 tracks
        let tracklist = create_test_tracklist(5);

        // Create Take component with parameters to take 0 from beginning
        let args = TakeArgs {
            limit: 0,
            from: "beginning".to_string(),
        };

        // Execute the component
        let result = Take::execute(&Client::default(), args, vec![tracklist.clone()]).unwrap();

        // Verify that we get an empty track list
        assert_eq!(result.len(), 0);
    }
}
