/// TrackList is a collection of FullTracks. It is used as a return type for source components.
pub type TrackList = Vec<rspotify::model::FullTrack>;

pub mod combiners;
pub mod conditionals;
pub mod filters;
pub mod sources;

use rspotify::AuthCodeSpotify as Client;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// NonExhaustive is a helper enum to allow us to Deserialize unknown components.
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
    type Args: JsonSchema;

    fn execute(client: &Client, args: Self::Args, prev: Vec<TrackList>) -> Result<TrackList>;
}

// --

macro_rules! components {
    ( $(( $name:literal, $comment:expr, $x:ident )),* ) => {
        /// The Component enum wraps all components with a tag-based deserializer.
        ///
        /// When being deserialized we look for an adjacent `component` tag, this tag allows
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
        #[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
        #[serde(tag = "component", content = "parameters")]
        #[schemars(description = "")]
        pub enum Component {
            $(
                // Map the component types to enum variants.
                // E.g. ArtistTopTracks(ArtistTopTracks::Args)
                #[serde(rename = $name)]
                #[doc = $comment]
                $x(<$x as Executable>::Args),
            )*
        }

        impl Component {

            /// Generate JSON schema for all components
            pub fn json_schema() -> schemars::Schema {
                schema_for!(Component)
            }

            /// Return the name of the component.
            pub fn name(&self) -> &'static str {
                match self {
                    $(Component::$x(_) => $name,)*
                }
            }

            /// Execute the component with the given arguments and previous component results.
            pub fn execute(self, client: &Client, prev: Vec<TrackList>) -> Result<TrackList> {
                match self {
                    $(Component::$x(args) => <$x>::execute(client, args, prev),)*
                }
            }
        }
    };
}

// Import component types
use self::filters::DeduplicateArtist;
use self::filters::DeduplicateTrack;
use self::filters::Take;
use self::sources::Album;
use self::sources::ArtistTopTracks;
use self::sources::UserLikedTracks;

#[rustfmt::skip::macros(components)]
components![
    // Sources
    ("source:artist_top_tracks", "Artist's top tracks", ArtistTopTracks),
    ("source:album", "Album tracks", Album),
    ("source:user_liked_tracks", "User liked tracks", UserLikedTracks),

    // Filters
    ("filter:take", "Take first N tracks", Take),
    ("filter:dedup_artist", "Deduplicate tracks by artist", DeduplicateArtist),
    ("filter:dedup_track", "Deduplicate tracks by ID", DeduplicateTrack)
];
