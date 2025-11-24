use chrono::DateTime;
use notify_rust::Notification;
use rss::{Channel, Item};
use std::{collections::HashSet, error::Error};

use crate::data::FeedLinkData;

pub mod config;
pub mod data;

pub async fn get_feed(link: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(link).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub fn is_item_unseen(pub_date: &str, last_seen: &str) -> Result<bool, chrono::ParseError> {
    let pub_date = pub_date.trim();
    let last_seen = last_seen.trim();
    let formatted_pub_date = DateTime::parse_from_rfc2822(pub_date)?;
    let formatted_last_seen = DateTime::parse_from_rfc2822(last_seen)?;

    Ok(formatted_pub_date > formatted_last_seen)
}

pub async fn check_items(items: &[Item], last_seen: &str) -> Result<Vec<Item>, Box<dyn Error>> {
    let mut unseen_items: Vec<Item> = vec![];

    for item in items {
        let pub_date = item.pub_date();
        match pub_date {
            Some(date) => {
                let is_unseen = is_item_unseen(date, last_seen);
                match is_unseen {
                    Ok(unseen) => {
                        if unseen {
                            unseen_items.push(item.clone());
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Someting went wrong: {}", e);
                    }
                }
            }
            None => {
                println!("Warning!: Item {} has no date.", item.link().unwrap());
            }
        }
    }

    Ok(unseen_items)
}

pub struct NotificationData {
    title: String,
    unseen_items_count: u64,
    latest_item: Item,
}

impl NotificationData {
    fn create_body(&self) -> String {
        let item_title = self.latest_item.title().unwrap_or("Untitled");

        format!("Latest Item: <i>{}</i>\nClick to read more!", item_title)
    }
    fn create_subject(&self) -> String {
        format!("{}, {} unread items!", self.title, self.unseen_items_count)
    }
    pub fn send_notify(&self) -> Result<(), Box<dyn Error>> {
        let subject = self.create_subject();
        let body = self.create_body();
        let link = self.latest_item.link().unwrap_or("");

        let mut notification = Notification::new();
        notification
            .summary(&subject)
            .body(&body)
            .action("default", "Open");

        let handle = notification.show()?;

        handle.wait_for_action(|action| match action {
            "default" => {
                if !link.is_empty() {
                    if let Err(e) = open::that(link) {
                        eprintln!("Failed to open link: {}", e);
                    }
                }
            }
            "__closed" => (),
            _ => (),
        });

        Ok(())
    }
}

pub async fn check_all_feeds_and_notify(
    feeds: &[FeedLinkData],
) -> Result<Vec<&str>, Box<dyn Error>> {
    let mut notifications: Vec<NotificationData> = Vec::new();
    let mut unseen_feeds: Vec<&str> = Vec::new();

    for feed in feeds {
        let feed_link = feed.feed_link();
        let channel = match get_feed(feed_link).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to fetch feed {}: {}", feed_link, e);
                continue;
            }
        };

        let items = channel.items();
        if items.is_empty() {
            continue;
        }

        let last_seen = feed.last_seen();
        let unseen = check_items(items, last_seen).await?;

        if unseen.is_empty() {
            continue;
        }

        unseen_feeds.push(feed.feed_link());
        let latest_item = unseen.last().unwrap().clone();

        notifications.push(NotificationData {
            title: channel.title().to_string(),
            unseen_items_count: unseen.len() as u64,
            latest_item,
        });
    }

    for notify in notifications {
        notify.send_notify()?;
    }

    Ok(unseen_feeds)
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
    use rss::ItemBuilder;

    #[test]
    fn test_is_item_unseen_true() {
        let item = ItemBuilder::default()
            .pub_date(String::from("Wed, 20 Nov 2024 10:00:00 +0000"))
            .build();

        let last_seen = "Wed, 20 Nov 2024 09:00:00 +0000";

        assert!(
            is_item_unseen(&item, last_seen).unwrap(),
            "Item should be unseen"
        );
    }

    #[test]
    fn test_is_item_unseen_false() {
        let item = ItemBuilder::default()
            .pub_date(String::from("Wed, 20 Nov 2024 08:00:00 +0000"))
            .build();

        let last_seen = "Wed, 20 Nov 2024 09:00:00 +0000";

        assert!(
            !is_item_unseen(&item, last_seen).unwrap(),
            "Item should be seen"
        );
    }

    #[tokio::test]
    async fn test_check_items() {
        let path = String::from("./test-check-items");

        let items = vec![
            ItemBuilder::default()
                .pub_date(String::from("Wed, 20 Nov 2024 08:00:00 +0000"))
                .build(),
            ItemBuilder::default()
                .pub_date(String::from("Wed, 20 Nov 2024 09:00:00 +0000"))
                .build(),
            ItemBuilder::default()
                .pub_date(String::from("Wed, 20 Nov 2024 10:00:00 +0000"))
                .build(),
        ];

        let last_seen = "Wed, 20 Nov 2024 08:30:00 +0000";
        let unseen = check_items(&items, last_seen).await.unwrap();

        assert_eq!(unseen.len(), 2, "Should return unseen items");
        assert_eq!(
            unseen.first().unwrap().pub_date().unwrap(),
            "Wed, 20 Nov 2024 09:00:00 +0000",
            "First unseen should be 9:00 after reverse"
        );

        std::fs::remove_dir_all(path).ok();
    }

    #[test]
    fn test_notification_create_strings() {
        let item = ItemBuilder::default()
            .title(String::from("Hello World"))
            .link(String::from("https://example.com"))
            .pub_date(String::from("Wed, 20 Nov 2024 11:00:00 +0000"))
            .build();

        let notif = NotificationData {
            title: String::from("My Feed"),
            unseen_items_count: 5,
            latest_item: item,
        };

        let subject = notif.create_subject();
        let body = notif.create_body();

        assert_eq!(subject, String::from("My Feed, 5 unread items!"));
        assert!(
            body.contains("Hello World"),
            "Body should contain the title"
        );
    }

    #[test]
    fn test_initiate_data_from_config_behavior() {
        let config_path = String::from("./test-initiate-config");
        let data_path = String::from("./test-initiate-config-data");

        // Create config
        {
            let mut config =
                config::Config::load(Some(&config_path)).expect("Failed to load config");

            config.clear();
            config.add_feed("link1", "* * * * *");
            config.add_feed("link2", "0 * * * *");
            config.save(Some(&config_path)).unwrap();
        }

        let config = config::Config::load(Some(&config_path)).unwrap();
        initiate_data_from_config(&config, Some(&data_path)).unwrap();

        let data = data::Data::load(Some(&data_path)).unwrap();
        let feeds = data.get_feeds();

        assert_eq!(feeds.len(), 2);
        assert!(feeds.contains(&String::from("link1")));
        assert!(feeds.contains(&String::from("link2")));

        std::fs::remove_dir_all(config_path).ok();
        std::fs::remove_dir_all(data_path).ok();
    }

    #[tokio::test]
    async fn test_check_all_feeds_inner_logic_mocked() {
        let item = ItemBuilder::default()
            .title(String::from("Brand New"))
            .link(String::from("https://example.com/new"))
            .pub_date(String::from("Wed, 20 Nov 2024 11:00:00 +0000"))
            .build();

        let channel = Channel {
            title: String::from("Mock Feed"),
            items: vec![item.clone()],
            ..Default::default()
        };

        let feed_data = FeedLinkData::new(
            String::from("mock://test"),
            String::from("* * * * *"),
            String::from("Wed, 20 Nov 2024 10:00:00 +0000"),
        );

        let unseen = check_items(channel.items(), feed_data.last_seen())
            .await
            .unwrap();

        assert_eq!(unseen.len(), 1);
        assert_eq!(unseen.first().unwrap().title().unwrap(), "Brand New");
    }

    #[tokio::test]
    #[ignore] // prevents cargo test from running it by default
    async fn test_actual_notification() {
        // Build a fake RSS item
        let mut item = Item::default();
        item.set_title(Some("Test Notification Title".into()));
        item.set_link(Some("https://example.com".into()));

        // Build NotificationData
        let notif = NotificationData {
            title: "Mock Feed".into(),
            unseen_items_count: 1,
            latest_item: item,
        };

        // This should trigger a real desktop notification.
        notif.send_notify().expect("Notification failed");

        // Keep test alive for a moment so user can see it
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
