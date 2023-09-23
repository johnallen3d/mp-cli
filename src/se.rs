use serde::ser::SerializeSeq;
use serde::Serializer;

use crate::song::Playlist;

pub fn serialize_playlists<S>(
    playlists: &Vec<Playlist>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(playlists.len()))?;
    for playlist in playlists {
        seq.serialize_element(&playlist.name)?;
    }
    seq.end()
}
