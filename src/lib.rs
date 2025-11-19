use chrono::Utc;
use html2text::from_read;
use notify_rust::Notification;
use rss::Channel;
use std::{collections::HashSet, error::Error};

pub mod config;
pub mod data;

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

pub fn initiate_data_from_config(
    config: &config::Config,
    data_path: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let mut data: data::Data = data::Data::load(data_path).expect("Failed to load or create data");

    let mut present_feeds: HashSet<String> = data.get_feeds().into_iter().collect();

    for feed in &config.feeds {
        if present_feeds.remove(&feed.link) {
            continue;
        };

        data.insert_link_map(&feed.link, &feed.schedule);
    }

    for obsolete in present_feeds {
        data.remove_link_map(&obsolete);
    }

    data.save(data_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_notify() {
        let feed_link = "https://archlinux.org/feeds/news/";
        let feed = get_feed(feed_link).await.unwrap();

        send_notify(&feed);
    }

    #[test]
    fn test_initiate_config() {
        let test_path = "./test-initiate-config";
        let mut config =
            config::Config::load(Some(test_path)).expect("Failed to load or create config");
        config.clear();
        config.add_feed("link1", "* * * * *");
        assert_eq!(config.feeds.len(), 1, "Should have 1 feed");
        config.save(Some(test_path)).unwrap();

        let data_path = "./test-initiate-config-data";
        initiate_data_from_config(&config, Some(data_path)).unwrap();
        let data = data::Data::load(Some(data_path)).unwrap();

        assert_eq!(data.get_feeds().len(), 1, "Should only have 1 feed");
        assert_eq!(
            data.get_feeds().first().unwrap(),
            "link1",
            "Should have link1"
        );

        data.save(Some(data_path)).unwrap();

        std::fs::remove_dir_all(test_path).ok();
        std::fs::remove_dir_all(data_path).ok();
    }
}
