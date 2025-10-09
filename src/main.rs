use rss_notify::config::Config;
use rss_notify::data::Data;
// use tokio::time::{Duration, Instant, sleep_until};
// use tokio_cron_scheduler::{Job, JobScheduler};

#[tokio::main]
async fn main() {
    let config: Config = Config::load(None).expect("Failed to load config");
    println!("Loaded config:\n {:#?}", config);
    let data: Data = Data::load(None).expect("Failed to load data");
    println!("Loaded data:\n {:#?}", data);
    // let scheduler = JobScheduler::new().await.unwrap();
    //     // cron job runs every minute
    // scheduler
    //     .add(
    //         Job::new_async("0 * * * * *", |_uuid, _locked| {
    //             Box::pin(async move {
    //                 let feed_link = "https://archlinux.org/feeds/news/";
    //                 let feed = get_feed(feed_link).await.unwrap();
    //
    //                 send_notify(&feed);
    //             })
    //         })
    //         .unwrap(),
    //     )
    //     .await
    //     .unwrap();
    //
    // // start scheduler
    // scheduler.start().await.unwrap();
    //
    // // keep running
    // loop {
    //     sleep_until(Instant::now() + Duration::from_secs(60)).await;
    // }
}
