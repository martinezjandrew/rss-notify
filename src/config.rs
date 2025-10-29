use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Feed {
    pub link: String,
    pub schedule: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub feeds: Vec<Feed>,
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

    pub fn clear(&mut self) {
        self.feeds.clear();
    }

    pub fn load(path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let path = get_config_path(path);

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let config = toml::from_str(&contents).expect("Failed to fit into the Config class");
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
    fs::write(path, toml_str)?;
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_feed_to_config() {
        let mut config: Config = Config::default();
        let url = "https://feeds.npr.org/1001/rss.xml";
        let schedule = "0/5 * * * * *";
        config.add_feed(url, schedule);
        assert_eq!(config.feeds[0].link, "https://feeds.npr.org/1001/rss.xml");
    }
    #[test]
    fn remove_feed_from_config() {
        let mut config: Config = Config::default();
        let url = "https://feeds.npr.org/1001/rss.xml";
        let schedule = "0/5 * * * * *";

        config.add_feed(url, schedule);
        assert_eq!(config.feeds.len(), 1);

        assert!(config.remove_feed(0).is_ok());
        assert_eq!(config.feeds.len(), 0);
    }
    #[test]
    fn list_feeds() {
        let mut config: Config = Config::default();
        let url = "https://feeds.npr.org/1001/rss.xml";
        let schedule = "0/5 * * * * *";
        config.add_feed(url, schedule);
        let output: String = config.list_feeds();
        assert_eq!(
            output.trim(),
            format!("0: {} - {}", url, schedule),
            "list feeds output mistmach"
        );
    }
    #[test]
    fn test_temp_config_path() {
        let test_path = "./test-temp-config-path";
        let path = get_config_path(Some(test_path));
        assert!(path.ends_with("config.toml"));

        std::fs::remove_dir_all(test_path).ok();
    }
    #[test]
    fn test_temp_config_file() {
        let test_path = "./test-temp-config-file";
        assert!(Config::load(Some(test_path)).is_ok());

        std::fs::remove_dir_all(test_path).ok();
    }
    #[test]
    fn add_to_and_remove_from_temp_config_file() {
        let test_path = "./test-add-to-and-remove-from-temp-config-file";
        let mut config = Config::load(Some(test_path)).expect("Failed to load or create config");
        config.clear();

        let url = "https://feeds.npr.org/1001/rss.xml";
        let schedule = "0/5 * * * * *";

        config.add_feed(url, schedule);
        assert_eq!(
            config.feeds.len(),
            1,
            "Config should have one feed after adding one feed."
        );

        config.save(Some(test_path)).expect("Failed to save config");

        let mut loaded_config =
            Config::load(Some(test_path)).expect("Failed to load config after save");

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
            .save(Some(test_path))
            .expect("Failed to save config after removal");

        let final_config =
            Config::load(Some(test_path)).expect("Failed to laod config after final save");

        assert_eq!(
            final_config.feeds.len(),
            0,
            "Final config should have 0 feeds"
        );

        std::fs::remove_dir_all(test_path).ok();
    }

    #[test]
    fn test_clear_feeds() {
        let test_path = "./test-clear-feeds";
        let mut config = Config::load(Some(test_path)).expect("Failed to load or create config");

        config.clear();

        assert_eq!(config.feeds.len(), 0, "Should be empty...");

        std::fs::remove_dir_all(test_path).ok();
    }
}
