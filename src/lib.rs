use chrono::Utc;
use html2text::from_read;
use notify_rust::Notification;
use rss::Channel;
use std::error::Error;

use crate::data::Data;

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

pub fn init() {}

pub fn initiate_data_from_config(config: config::Config) {
    let mut data: Data = Data::load(Some("./test-data")).expect("Failed to load or create data");
    let data_feeds = data.get_feeds();
    for feed in config.feeds {
        if data_feeds.contains(&feed.link) {
            continue;
        };
        data.update_last_seen(&feed.link);
    }
}

pub struct UnseenItems {
    date: String,
    title: String,
    link: String,
}
pub fn get_unseen_items(channel: &Channel) -> Result<Vec<UnseenItems>, Box<dyn Error>> {
    // get last seen date from data file
    // initiate array to hold unseen items in tuples: date, title, link
    // loop through channel items
    //     stop if item date is on or before last seen date
    //     add item to unseen items array
    //  return list of unseen items
    let data = Data::load(None)?;
    let link = &channel.link;
    let last_seen = data.get_last_seen(link);
    if last_seen.is_empty() {
        panic!("No last_seen found.")
    };
    let mut unseen_items: Vec<UnseenItems> = vec![];
    for item in &channel.items {
        let pub_date = item.pub_date().unwrap_or("");
        if *pub_date != last_seen {
            break;
        } else {
            let unseen = UnseenItems {
                date: pub_date.to_string(),
                title: item.title().unwrap_or("").to_string(),
                link: item.link().unwrap_or("").to_string(),
            };
            unseen_items.insert(0, unseen);
        };
    }
    Ok(unseen_items)
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
}
