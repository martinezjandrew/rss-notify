use chrono::Utc;
use directories::ProjectDirs;
use html2text::from_read;
use notify_rust::Notification;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Feed {
    link: String,
    schedule: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    feeds: Vec<Feed>,
}

impl Config {
    pub fn add_feed(&mut self, url: &str, schedule: &str) {
        let new_feed = Feed {
            link: url.to_string(),
            schedule: schedule.to_string(),
        };
        self.feeds.push(new_feed);
    }
    pub fn remove_feed(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.feeds.len() {
            Err("Feed index is out of bounds.")
        } else {
            self.feeds.remove(index);
            Ok(())
        }
    }
    pub fn list_feeds(&self) -> String {
        let feed_iter = self.feeds.iter();

        let mut output = String::new();

        for (i, feed) in feed_iter.enumerate() {
            let line = format!("{}: {} - {}", i, &feed.link, &feed.schedule);
            output.push_str(&line);
            output.push('\n');
        }

        output
    }

    pub fn load(path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let path = get_config_path(path);

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Config::default();
            create_config(&path, &config)?;
            Ok(config)
        }
    }

    pub fn save(&self, path: Option<&str>) -> Result<(), Box<dyn Error>> {
        let path = get_config_path(path);

        if path.exists() {
            let toml_file = toml::to_string_pretty(self)?;
            fs::write(&path, toml_file)?;
        } else {
            create_config(&path, self)?;
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { feeds: vec![] }
    }
}

fn get_config_path(path: Option<&str>) -> PathBuf {
    if let Some(p) = path {
        let dir = PathBuf::from(p);
        fs::create_dir_all(&dir).expect("Failed to create config directory.");
        return dir.join("config.toml");
    }

    let dirs = ProjectDirs::from("com", "martinezjandrew", "rss-notify")
        .expect("Unable to get config directory.");

    let config_dir = dirs.config_dir();
    fs::create_dir_all(config_dir).expect("Failed to create config directory.");

    config_dir.join("config.toml")
}

fn create_config(path: &Path, config: &Config) -> Result<PathBuf, Box<dyn Error>> {
    if path.exists() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Config path already exists.",
        )));
    }

    let toml_str = toml::to_string_pretty(&config)?;
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
    fn add_feed_to_config() {
        let mut config: Config = Config::default();
        let url = "https://feeds.npr.org/1001/rss.xml";
        let schedule = "0/5 * * * * *";
        config.add_feed(&url, &schedule);
        assert_eq!(config.feeds[0].link, "https://feeds.npr.org/1001/rss.xml");
    }
    #[test]
    fn remove_feed_from_config() {
        let mut config: Config = Config::default();
        let url = "https://feeds.npr.org/1001/rss.xml";
        let schedule = "0/5 * * * * *";
        config.add_feed(&url, &schedule);
        assert_eq!(config.feeds.len(), 1);
        config.remove_feed(0);
        assert_eq!(config.feeds.len(), 0);
    }
    #[test]
    fn list_feeds() {
        let mut config: Config = Config::default();
        let url = "https://feeds.npr.org/1001/rss.xml";
        let schedule = "0/5 * * * * *";
        config.add_feed(&url, &schedule);
        let output: String = config.list_feeds();
        assert_eq!(
            output.trim(),
            format!("0: {} - {}", url, schedule),
            "list feeds output mistmach"
        );
    }
    #[test]
    fn test_temp_config_path() {
        let path = get_config_path(Some("./test-config"));
        assert!(path.ends_with("config.toml"));
    }
    #[test]
    fn test_temp_config_file() {
        let config: Config =
            Config::load(Some("./test-config")).expect("Failed to load or create config");
        assert_eq!(config.feeds.len(), 0, "New config should have no feeds");
    }
    #[test]
    fn add_to_and_remove_from_temp_config_file() {
        let test_path = "./test-config";
        let mut config = Config::load(Some(&test_path)).expect("Failed to load or create config");

        let url = "https://feeds.npr.org/1001/rss.xml";
        let schedule = "0/5 * * * * *";

        config.add_feed(url, schedule);
        assert_eq!(
            config.feeds.len(),
            1,
            "Config should have one feed after adding one feed."
        );

        config
            .save(Some(&test_path))
            .expect("Failed to save config");

        let mut loaded_config =
            Config::load(Some(&test_path)).expect("Failed to load config after save");

        assert_eq!(
            loaded_config.feeds.len(),
            1,
            "Loaded config should have one feed"
        );

        loaded_config.remove_feed(0).expect("Failed to remove feed");

        assert_eq!(
            loaded_config.feeds.len(),
            0,
            "Config should have 0 feeds after removal"
        );

        loaded_config
            .save(Some(&test_path))
            .expect("Failed to save config after removal");

        let final_config =
            Config::load(Some(&test_path)).expect("Failed to laod config after final save");

        assert_eq!(
            final_config.feeds.len(),
            0,
            "Final config should have 1 feed"
        );
    }
}
