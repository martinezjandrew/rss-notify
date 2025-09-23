use std::error::Error;
use rss::Channel;
use notify_rust::Notification;
use chrono::Utc;
use html2text::from_read;
use serde::Deserialize;
use toml;

pub async fn get_feed(link: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(link)
        .await?
        .bytes()
        .await?;
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

    println!("Executed notification for \"{feed_title}\" at {time}",
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
            _ => ()
        });


}

fn create_body(title: &str, description: &str) -> String {
    let mut plain = from_read(description.as_bytes(), 80).unwrap();
    if plain.len() > 150 {
        plain.truncate(150);
        plain.push_str("...");
    }

    format!(
        "~~<i>{}</i>~~\n\n{}\n\nClick to read more 👉",
        title, plain
    )
}

#[derive(Debug, Deserialize)]
pub struct Feed {
    link: String,
    schedule: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    feeds: Vec<Feed>,
}

pub fn load() -> Config {
    let config: Config = toml::from_str(r#"
        feeds = [{link = "https://archlinux.org/feeds/news/",schedule = "0/5 * * * * *"}]
    "#).unwrap();

    config
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
    fn load_config() {
        let config: Config = load();

        assert_eq!(config.feeds[0].link, "https://archlinux.org/feeds/news/");
        assert_eq!(config.feeds[0].schedule, "0/5 * * * * *");
    }
}
