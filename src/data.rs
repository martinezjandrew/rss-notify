use chrono::Local;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

// url to feed
pub type FeedLink = String;

// date of last seen item from feed in rfc 2822 format
pub type Date = String;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Data {
    last_seen: HashMap<FeedLink, Date>,
}

impl Data {
    pub fn get_last_seen(self, feed: &str) -> String {
        match self.last_seen.get(feed) {
            Some(val) => val.to_string(),
            None => String::new(),
        }
    }

    pub fn update_last_seen(&mut self, feed: &str) {
        let feed = String::from_str(feed).unwrap();
        let date = Local::now().to_rfc2822();
        self.last_seen.insert(feed, date);
    }

    pub fn remove_last_seen(&mut self, feed: &str) {
        self.last_seen.remove(feed);
    }

    pub fn get_feeds(&self) -> Vec<String> {
        self.last_seen.keys().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.last_seen.clear();
    }

    pub fn load(path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let path = get_data_path(path);

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let data = toml::from_str(&contents)?;
            Ok(data)
        } else {
            let data = Data::default();
            create_data(&path, &data)?;
            Ok(data)
        }
    }

    pub fn save(&self, path: Option<&str>) -> Result<(), Box<dyn Error>> {
        let path = get_data_path(path);

        if path.exists() {
            let toml_file = toml::to_string_pretty(self)?;
            fs::write(&path, toml_file)?;
        } else {
            create_data(&path, self)?;
        }
        Ok(())
    }
}

fn get_data_path(path: Option<&str>) -> PathBuf {
    if let Some(p) = path {
        let dir = PathBuf::from(p);
        fs::create_dir_all(&dir).expect("Failed to create data directory.");
        return dir.join("data.toml");
    }

    let dirs = ProjectDirs::from("com", "martinezjandrew", "rss-notify")
        .expect("Unable to get data directory.");

    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir).expect("Failed to create config directory.");

    data_dir.join("data.toml")
}

fn create_data(path: &Path, data: &Data) -> Result<PathBuf, Box<dyn Error>> {
    if path.exists() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Config path already exists.",
        )));
    }

    let toml_str = toml::to_string_pretty(&data)?;
    fs::write(path, toml_str)?;
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_data_path() {
        let path = get_data_path(Some("./test-data"));
        assert!(path.ends_with("data.toml"));
    }

    #[test]
    fn test_insert_last_seen() {
        let mut data: Data =
            Data::load(Some("./test-data")).expect("Failed to load or create data");
        data.clear();
        data.update_last_seen("hello");
        assert_eq!(data.last_seen.len(), 1, "Just inserted a feed, should be 1");
        assert_eq!(
            data.get_last_seen("hello"),
            Local::now().to_rfc2822(),
            "Should be the date"
        );
    }

    #[test]
    fn test_insert_and_save_to_data() {
        let mut data: Data =
            Data::load(Some("./test-data")).expect("Failed to load or create data");
        data.clear();
        data.update_last_seen("hello");
        data.save(Some("./test-data")).expect("failed to save data");

        let data2: Data = Data::load(Some("./test-data")).expect("Failed to load or create data");
        assert_eq!(
            data2.last_seen.len(),
            1,
            "Inserted a feed before save, should be 1"
        );
    }

    #[test]
    fn test_clear_data() {
        let mut data: Data =
            Data::load(Some("./test-data")).expect("Failed to load or create data");
        data.clear();
        data.update_last_seen("hello");
        assert_eq!(
            data.last_seen.len(),
            1,
            "Inserted a feed before save, should be 1"
        );
        data.clear();
        assert!(data.last_seen.is_empty(), "Should be empty after clearing");
    }

    #[test]
    fn test_get_feeds() {
        let mut data: Data =
            Data::load(Some("./test-data")).expect("Failed to load or create data");
        data.clear();
        data.update_last_seen("hello");
        let feeds = data.get_feeds();
        assert!(!feeds.is_empty(), "Should get back one feed");
    }

    #[test]
    fn test_remove_feed() {
        let mut data: Data =
            Data::load(Some("./test-data")).expect("Failed to load or create data");
        data.clear();
        data.update_last_seen("hello");
        let feeds = data.get_feeds();
        assert!(!feeds.is_empty(), "Should get back one feed");
        data.remove_last_seen("hello");
        let feeds = data.get_feeds();
        assert!(feeds.is_empty(), "Should get empty now");
    }
}
