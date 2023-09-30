use serde::ser::SerializeSeq;
use serde::Serializer;

use crate::song::Playlist;
use crate::time::Time;

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

pub fn serialize_time<S>(time: &Time, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&time.as_string)
}
