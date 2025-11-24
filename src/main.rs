use std::env;

use rss_notify::check_all_feeds_and_notify;
// use rss_notify::config::Config;
use rss_notify::data::Data;

#[derive(Debug)]
enum ArgumentOptions {
    Check,
    Add,
    Remove,
    List,
    Help,
}

impl ArgumentOptions {
    async fn execute(&self, args: &[String], data: &mut Data) {
        match self {
            ArgumentOptions::Check => run_check(data).await.expect("WHAT"),
            ArgumentOptions::Add => run_add(args, data).expect("WHAT"),
            ArgumentOptions::Remove => run_remove(args, data).expect("WHAT"),
            ArgumentOptions::List => run_list(data),
            ArgumentOptions::Help => run_help(),
        }
    }
}

async fn run_check(data: &mut Data) -> Result<(), String> {
    let feed_link_data_list = data.get_all_feed_link_data();
    let unseen_feeds = check_all_feeds_and_notify(&feed_link_data_list).await;

    let unseen_feeds: Vec<&str> = match unseen_feeds {
        Ok(feeds) => feeds,
        Err(e) => {
            return Err(format!("Error Checking: {}", e));
        }
    };

    for feed in &unseen_feeds {
        data.update_link_map(feed);
    }

    data.save(None).expect("Didn't save...");
    Ok(())
}

fn run_list(data: &Data) {
    let data_map = data.link_map();

    for feed_link_data in data_map.keys() {
        println!("{}", feed_link_data);
    }
}

fn run_add(args: &[String], data: &mut Data) -> Result<(), String> {
    let link = args.get(2).ok_or("No link provided")?.clone();
    let frequency = args.get(3).ok_or("No frequency provided")?.clone();

    data.insert_link_map(&link, &frequency);
    data.save(None).expect("Didn't save...");
    Ok(())
}

fn run_remove(args: &[String], data: &mut Data) -> Result<(), String> {
    let link = args.get(2).ok_or("No link provided")?.clone();

    data.remove_link_map(&link);
    data.save(None).expect("Didn't save...");
    Ok(())
}

fn run_help() {
    println!(
        r#"Available commands:

        check
            Checks if any subscribed feed has unseen items.

        list
            View all subscribed feeds.

        add <link> <frequency>
            Add a new feed to the config.
            Arguments:
                link    The RSS/Atom feed URL.
                frequency   How often to check the feed in cron-like format (e.g., \"* * * * *\")

        remove <link>
            Remove a feed from the config.
            Arguments:
                link    The RSS/Atom feed URL."#
    )
}

struct Argument {
    specified_option: ArgumentOptions,
}

impl Argument {
    fn new(args: &[String]) -> Result<Self, String> {
        if args.len() < 2 {
            return Err("No command provided".into());
        }

        let arg = args[1].as_str();

        let option = match arg {
            "check" => ArgumentOptions::Check,
            "add" => ArgumentOptions::Add,
            "remove" => ArgumentOptions::Remove,
            "list" => ArgumentOptions::List,
            "help" => ArgumentOptions::Help,
            _ => return Err(format!("Invalid command: {}", arg)),
        };

        Ok(Argument {
            specified_option: option,
        })
    }
}

#[tokio::main]
async fn main() {
    // let config: Config = Config::load(None).expect("Failed to load config");
    let mut data: Data = Data::load(None).expect("Failed to load data");

    let args: Vec<String> = env::args().collect();

    let argument = Argument::new(&args).expect("Invalid command");

    argument.specified_option.execute(&args, &mut data).await;
}
