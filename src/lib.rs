use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use html2text::from_read;
use notify_rust::Notification;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

pub async fn get_feed(link: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(link).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub fn send_notify(channel: &Channel) {
    let feed_title = channel.title();
    let latest_item = channel.items().get(0);
    let title = latest_item.expect("REASON").title().unwrap();
    let description = latest_item.expect("REASON").description().unwrap();
    let body = create_body(&feed_title, &description);
    let link = latest_item.expect("REASON").link().unwrap();

    println!(
        "Executed notification for \"{feed_title}\" at {time}",
        feed_title = feed_title,
        time = Utc::now()
    );

    Notification::new()
        .summary(title)
        .body(&body)
        .action("default", "default")
        .show()
        .unwrap()
        .wait_for_action(|action| match action {
            "default" => open::that(link).expect("REASON"),
            "__closed" => println!("the notification was closed"),
            _ => (),
        });
}

fn create_body(title: &str, description: &str) -> String {
    let mut plain = from_read(description.as_bytes(), 80).unwrap();
    if plain.len() > 150 {
        plain.truncate(150);
        plain.push_str("...");
    }

    format!("~~<i>{}</i>~~\n\n{}\n\nClick to read more 👉", title, plain)
}

pub mod config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    last_seen: HashMap<String, String>,
}

impl Data {
    pub fn get_last_seen(self, feed: &str) -> String {
        match self.last_seen.get(feed) {
            Some(val) => val.to_string(),
            None => String::new(),
        }
    }
    pub fn update_last_seen(&mut self, feed: &str, date: &str) {
        let feed = String::from_str(feed).unwrap();
        let date = String::from_str(date).unwrap();
        self.last_seen.insert(feed, date);
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
impl Default for Data {
    fn default() -> Self {
        Self {
            last_seen: HashMap::new(),
        }
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
    fs::write(&path, toml_str)?;
    Ok(path.to_path_buf())
}
// function will update the "last_read" item for a feed
pub fn save_data() -> String {
    let mut data = String::new();
    data.push_str("NOT IMPLEMENTED YET!");
    data
}

// function will grab the "last_read" item from a feed
pub fn load_data() -> String {
    let mut data = String::new();
    data.push_str("NOT IMPLEMENTED YET!");
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_send_notify() {
        let feed_link = "https://archlinux.org/feeds/news/";
        let feed = get_feed(feed_link).await.unwrap();

        send_notify(&feed);
    }

    #[test]
    fn test_temp_data_path() {
        let path = get_data_path(Some("./test-data"));
        assert!(path.ends_with("data.toml"));
    }
    #[test]
    fn test_insert_last_seen() {
        let mut data: Data =
            Data::load(Some("./test-data")).expect("Failed to load or create data");
        data.update_last_seen("hello", "world");
        assert_eq!(data.last_seen.len(), 1, "Just inserted a feed, should be 1");
        assert_eq!(data.get_last_seen("hello"), "world", "Should be 'world'");
    }
    #[test]
    fn test_insert_and_save_to_data() {
        let mut data: Data =
            Data::load(Some("./test-data")).expect("Failed to load or create data");
        data.update_last_seen("hello", "world");
        data.save(Some("./test-data")).expect("failed to save data");

        let data2: Data = Data::load(Some("./test-data")).expect("Failed to load or create data");
        assert_eq!(
            data2.last_seen.len(),
            1,
            "Inserted a feed before save, should be 1"
        );
    }
}
