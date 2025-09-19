use tokio::time::{sleep_until, Instant, Duration};
use tokio_cron_scheduler::{Job, JobScheduler};
use rss_notify::{get_feed, send_notify};

#[tokio::main]
async fn main() {

    let scheduler = JobScheduler::new().await.unwrap();
        // cron job runs every minute
    scheduler
        .add(
            Job::new_async("0 * * * * *", |_uuid, _locked| {
                Box::pin(async move {
                    let feed_link = "https://archlinux.org/feeds/news/";
                    let feed = get_feed(feed_link).await.unwrap();

                    send_notify(&feed);
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
