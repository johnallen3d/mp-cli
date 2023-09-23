use std::{fmt, fs, path::Path};

use serde::Serialize;

use crate::{se::serialize_playlists, status::Status};

const VALID_EXTENSIONS: &[&str] = &[
    "mp3", "ogg", "flac", "wav", "aac", "m4a", "wma", "opus", "dffs", "dsf",
    "ape", "tta",
];

#[derive(Serialize)]
pub struct Song {
    pub inner: mpd::song::Song,
}

#[derive(Serialize)]
pub struct Current {
    artist: String,
    title: String,
}

#[derive(Serialize)]
pub struct TrackList {
    pub songs: Vec<Current>,
}

impl fmt::Display for TrackList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, song) in self.songs.iter().enumerate() {
            write!(f, "{index}={song}")?;
        }

        Ok(())
    }
}

impl From<Status> for Current {
    fn from(status: Status) -> Self {
        Current {
            artist: status.artist,
            title: status.title,
        }
    }
}

impl From<Song> for Current {
    fn from(song: Song) -> Self {
        Current {
            artist: song.inner.artist.unwrap_or(String::new()),
            title: song.inner.title.unwrap_or(String::new()),
        }
    }
}

impl fmt::Display for Current {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} - {}", self.artist, self.title)
    }
}

#[derive(Serialize)]
pub struct Playlists {
    #[serde(serialize_with = "serialize_playlists")]
    pub playlists: Vec<Playlist>,
}

#[derive(Default, Serialize)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
}

impl From<String> for Playlist {
    fn from(name: String) -> Self {
        Playlist {
            name,
            ..Default::default()
        }
    }
}

impl fmt::Display for Playlists {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, playlist) in self.playlists.iter().enumerate() {
            write!(f, "{}={}", index, playlist.name)?;
        }

        Ok(())
    }
}

pub struct File {
    pub full_path: String,
    pub relative_path: String,
}

pub struct Finder {
    music_dir: String,
    pub found: Vec<File>,
}

impl Finder {
    pub fn new(music_dir: String) -> Self {
        Finder {
            music_dir,
            found: Vec::<File>::new(),
        }
    }

    fn is_music_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return VALID_EXTENSIONS.contains(&ext_str);
            }
        }
        false
    }

    fn file_for(&self, path: &Path) -> Option<File> {
        let full_path = path.to_str().unwrap_or("").to_string();
        let mut relative_path = match full_path.strip_prefix(&self.music_dir) {
            Some(remainder) => remainder.to_string(),
            None => full_path.clone(),
        };

        if relative_path.starts_with('/') {
            relative_path.remove(0);
        }

        if Self::is_music_file(full_path.as_ref()) {
            Some(File {
                full_path,
                relative_path,
            })
        } else {
            None
        }
    }

    pub fn find(&mut self, file_or_dir: &Path) -> eyre::Result<()> {
        if file_or_dir.is_dir() {
            for entry in fs::read_dir(file_or_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    self.find(&path)?;
                } else if let Some(song) = self.file_for(&path) {
                    self.found.push(song);
                }
            }
        } else if let Some(song) = self.file_for(file_or_dir) {
            self.found.push(song);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File as StdFile;
    use std::io::Write;
    use std::path::PathBuf;
    use tempdir::TempDir;

    fn create_temp_music_files(temp_dir: &Path) {
        let music_files = ["song1.mp3", "song2.flac", "song3.ogg"];
        for music_file in &music_files {
            let file_path = temp_dir.join(music_file);
            let mut temp_file = StdFile::create(file_path).unwrap();
            temp_file.write_all(b"dummy content").unwrap();
        }
    }

    #[test]
    fn test_is_music_file() {
        let valid_file = Path::new("test.mp3");
        let invalid_file = Path::new("test.txt");

        assert!(Finder::is_music_file(valid_file));
        assert!(!Finder::is_music_file(invalid_file));
    }

    #[test]
    fn test_find() {
        let temp_dir = TempDir::new("music").unwrap();
        create_temp_music_files(temp_dir.path());

        let music_dir_str = temp_dir.path().to_str().unwrap().to_string();
        let mut finder = Finder::new(music_dir_str.clone());

        let result = finder.find(&PathBuf::from(&music_dir_str));

        assert!(result.is_ok());
        assert_eq!(finder.found.len(), 3); // Should find 3 music files
    }

    #[test]
    fn test_file_for() {
        let temp_dir = TempDir::new("music").unwrap();
        let music_file = "song.mp3";
        let file_path = temp_dir.path().join(music_file);

        let mut temp_file = StdFile::create(file_path.clone()).unwrap();
        temp_file.write_all(b"dummy content").unwrap();

        let music_dir_str = temp_dir.path().to_str().unwrap().to_string();
        let finder = Finder::new(music_dir_str);

        let file = finder.file_for(&file_path).unwrap();

        assert_eq!(file.full_path, file_path.to_str().unwrap());
        assert_eq!(file.relative_path, music_file);
    }
}
