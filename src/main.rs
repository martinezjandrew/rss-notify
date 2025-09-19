use chrono::Utc;
use tokio::time::{sleep_until, Instant, Duration};
use tokio_cron_scheduler::{Job, JobScheduler};
use notify_rust::Notification;
use std::error::Error;
use rss::Channel;
use open;

async fn example_feed() -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get("https://archlinux.org/feeds/news/")
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

#[tokio::main]
async fn main() {

    let scheduler = JobScheduler::new().await.unwrap();
        // cron job runs every minute
    scheduler
        .add(
            Job::new_async("0 * * * * *", |_uuid, _locked| {
                Box::pin(async move {
                    let feed = example_feed().await.unwrap();
                    let latest_item = feed.items().get(0);

                    let title = latest_item.expect("REASON").title().unwrap();
                    let description = latest_item.expect("REASON").description().unwrap();
                    let link = latest_item.expect("REASON").link().unwrap();



                    println!("Task executed at: {}", Utc::now());
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
                })
            })
            .unwrap(),
        )
        .await
        .unwrap();

    // start scheduler
    scheduler.start().await.unwrap();

    // keep running
    loop {
        sleep_until(Instant::now() + Duration::from_secs(60)).await;
    }
}
