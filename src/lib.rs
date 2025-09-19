use std::error::Error;
use rss::Channel;
use notify_rust::Notification;
use chrono::Utc;

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
    let link = latest_item.expect("REASON").link().unwrap();

    println!("Executed notification for \"{feed_title}\" at {time}",
        feed_title = feed_title,
        time = Utc::now() 
    );

    Notification::new()
        .summary(title)
        .body(description)
        .action("default", "default")
        .show()
        .unwrap()
        .wait_for_action(|action| match action {
            "default" => open::that(link).expect("REASON"),
            "__closed" => println!("the notification was closed"),
            _ => ()
    });


}
