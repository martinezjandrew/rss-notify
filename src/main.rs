use rss_notify::config::Config;
use rss_notify::data::Data;

#[tokio::main]
async fn main() {
    let config: Config = Config::load(None).expect("Failed to load config");
    println!("Loaded config:\n {:#?}", config);
    let data: Data = Data::load(None).expect("Failed to load data");
    println!("Loaded data:\n {:#?}", data);
}
