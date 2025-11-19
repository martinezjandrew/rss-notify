use chrono::Local;
use chrono::prelude::DateTime;
use croner::Cron;
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
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FeedLinkData {
    feed_link: FeedLink,
    frequency: String,
    last_seen: String,
}

impl FeedLinkData {
    pub fn update_last_seen(&mut self) {
        let date = Local::now().to_rfc2822();
        self.last_seen = date
    }

    pub fn is_frequency_check_due(self) -> bool {
        let cron = Cron::from_str(&self.frequency).expect("Should work....");
        let seen_date = DateTime::parse_from_rfc2822(&self.last_seen).unwrap();
        let next = cron.find_next_occurrence(&seen_date, false).unwrap();

        let now = Local::now();

        now >= next
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Data {
    link_map: HashMap<FeedLink, FeedLinkData>,
}

impl Data {
    pub fn get_link_map(&self, feed: &str) -> Option<FeedLinkData> {
        self.link_map.get(feed).cloned()
    }

    pub fn insert_link_map(&mut self, feed_link: &str, frequency: &str) -> Option<&FeedLinkData> {
        let feed_link_data = FeedLinkData {
            feed_link: String::from(feed_link),
            frequency: String::from(frequency),
            last_seen: String::new(),
        };

        self.link_map
            .insert(String::from(feed_link), feed_link_data);

        self.link_map.get(feed_link)
    }

    pub fn update_link_map(&mut self, feed: &str) -> Option<&FeedLinkData> {
        let feed_link_data = self.link_map.get_mut(feed);

        if let Some(data) = feed_link_data {
            data.update_last_seen();
            self.link_map.get(feed)
        } else {
            None
        }
    }

    pub fn remove_link_map(&mut self, feed: &str) {
        self.link_map.remove(feed);
    }

    pub fn get_feeds(&self) -> Vec<String> {
        self.link_map.keys().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.link_map.clear();
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
    use chrono::Days;

    use super::*;

    #[test]
    fn test_temp_data_path() {
        let data_path = "./test-temp-data-path";
        let path = get_data_path(Some(data_path));
        assert!(path.ends_with("data.toml"));

        std::fs::remove_dir_all(data_path).ok();
    }

    #[test]
    fn test_insert_link_map() {
        let path = "./test-insert-last-seen";
        let mut data: Data = Data::load(Some(path)).expect("Failed to load or create data");

        let _ = data.insert_link_map("https://test/", "* * 10 * *");

        assert_eq!(data.link_map.len(), 1, "Just inserted a feed, should be 1");

        std::fs::remove_dir_all(path).ok();
    }

    #[test]
    fn test_insert_and_save_to_data() {
        let path = "./test-insert-and-save-to-data";
        let mut data: Data = Data::load(Some(path)).expect("Failed to load or create data");

        let _ = data.insert_link_map("https://test/", "* * 10 * *");

        data.save(Some(path)).expect("failed to save data");

        let data2: Data = Data::load(Some(path)).expect("Failed to load or create data");
        assert_eq!(
            data2.link_map.len(),
            1,
            "Inserted a feed before save, should be 1"
        );

        std::fs::remove_dir_all(path).ok();
    }

    #[test]
    fn test_clear_data() {
        let path = "./test-clear-data";
        let mut data: Data = Data::load(Some(path)).expect("Failed to load or create data");

        let _ = data.insert_link_map("https://test/", "* * 10 * *");

        assert_eq!(
            data.link_map.len(),
            1,
            "Inserted a feed before save, should be 1"
        );
        data.clear();
        assert!(data.link_map.is_empty(), "Should be empty after clearing");

        std::fs::remove_dir_all(path).ok();
    }

    #[test]
    fn test_get_feeds() {
        let path = "./test-get-feeds";
        let mut data: Data = Data::load(Some(path)).expect("Failed to load or create data");

        let _ = data.insert_link_map("https://test/", "* * 10 * *");
        let feeds = data.get_feeds();
        assert!(!feeds.is_empty(), "Should get back one feed");

        std::fs::remove_dir_all(path).ok();
    }

    #[test]
    fn test_remove_feed() {
        let path = "./test-remove-feed";
        let mut data: Data = Data::load(Some(path)).expect("Failed to load or create data");

        let _ = data.insert_link_map("https://test/", "* * 10 * *");
        let feeds = data.get_feeds();
        assert!(!feeds.is_empty(), "Should get back one feed");

        data.remove_link_map("https://test/");
        let feeds = data.get_feeds();
        assert!(feeds.is_empty(), "Should get empty now");

        std::fs::remove_dir_all(path).ok();
    }

    #[test]
    fn test_if_time_to_check() {
        let now = Local::now();
        let sample_last_seen = now.checked_sub_days(Days::new(10));
        let sample = FeedLinkData {
            feed_link: String::from("https://test.com/"),
            frequency: String::from("* * 10 * *"),
            last_seen: sample_last_seen.unwrap().to_rfc2822(),
        };
        assert!(
            sample.is_frequency_check_due(),
            "Today is 10 days from 10 days ago..."
        );
    }
}
