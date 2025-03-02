/// TrackList is a collection of FullTracks. It is used as a return type for source components.
pub type TrackList = Vec<rspotify::model::FullTrack>;

pub mod combiners;
pub mod conditinals;
pub mod filters;
pub mod sources;

use rspotify::AuthCodeSpotify as Client;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// NonExhaustive is a helper enum to allow us to Deserialze unknown components.
/// Required as a workaround due to `#[serde(other)]` not working with tuple variants.
///
/// Ref: <https://github.com/serde-rs/serde/issues/1701#issuecomment-584677088>
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum NonExhaustive<T> {
    Known(T),
    Unknown(serde_json::Value),
}

impl<T> NonExhaustive<T> {
    pub fn unwrap(self) -> T {
        match self {
            NonExhaustive::Known(inner) => inner,
            NonExhaustive::Unknown(_) => panic!("unknown component"),
        }
    }
}

/// The Executable Trait should be implemented by all components.
///
/// Each Executable component should take an arguments object, as well as a list of previous
/// component outputs, and return a single [`TrackList`].
pub trait Executable {
    type Args;

    fn execute(client: &Client, args: Self::Args, prev: Vec<TrackList>) -> Result<TrackList>;
}

// --

macro_rules! components {
    ( $(( $a:literal, $b:ident )),* ) => {
        /// The Component enum wraps all components with a tag-based deserializer.
        ///
        /// When being deserialized we look for an adjancent `component` tag, this tag allows
        /// us to map the `parameters` into the correct component `Args`.
        ///
        /// **Example**
        ///
        /// ```rust
        /// let yaml = r#"
        ///   - component: "source:artist_top_tracks"
        ///     parameters:
        ///       id: "spotify:artist:6qqNVTkY8uBg9cP3Jd7DAH"
        ///   # ...
        /// "#;
        ///
        /// let components: Vec<Component> = serde_yaml::from_str(yaml);
        /// ```
        #[derive(Deserialize, Serialize, Clone, Debug)]
        #[serde(tag = "component", content = "parameters")]
        pub enum Component {
            $(
                // Map the component types to enum varients.
                // E.g. ArtistTopTracks(ArtistTopTracks::Args)
                #[serde(rename = $a)]
                $b(<$b as Executable>::Args),
            )*
        }

        impl Component {
            /// Return the name of the component.
            pub fn name(&self) -> &'static str {
                match self {
                    $(Component::$b(_) => $a,)*
                }
            }

            /// Execute the component with the given arguments and previous component results.
            pub fn execute(self, client: &Client, prev: Vec<TrackList>) -> Result<TrackList> {
                match self {
                    $(Component::$b(args) => <$b>::execute(client, args, prev),)*
                }
            }
        }
    };
}

// Import component types
use self::sources::ArtistTopTracks;
use self::sources::Album;
use self::sources::UserLikedTracks;
use self::filters::Take;
use self::filters::DeduplicateArtist;
use self::filters::DeduplicateTrack;

#[rustfmt::skip::macros(components)]
components![
    // Sources
    ("source:artist_top_tracks", ArtistTopTracks),
    ("source:album", Album),
    ("source:user_liked_tracks", UserLikedTracks),

    // Filters
    ("filter:take", Take),
    ("filter:dedup_artist", DeduplicateArtist),
    ("filter:dedup_track", DeduplicateTrack)
];