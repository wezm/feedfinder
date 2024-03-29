use feedfinder;

use std::io::Read;
use url::Url;

// This example can be used to list the feeds found at a URL by combining it with
// curl. For example:
//
// URL=https://www.wezm.net/v2/ ; curl "$URL" | cargo run --example cli "$URL"
fn main() {
    for arg in std::env::args().skip(1).take(1) {
        let url = Url::parse(&arg).expect("unable to parse URL");

        // Read html from stdin
        let mut html = String::new();
        std::io::stdin()
            .read_to_string(&mut html)
            .expect("error reading HTML from stdin");

        match feedfinder::detect_feeds(&url, &html) {
            Ok(feeds) => {
                println!("Possible feeds for {}", url);
                for feed in feeds {
                    println!(
                        "title: {}\nurl: {}\ntype: {:?}\n",
                        feed.title().unwrap_or_default(),
                        feed.url(),
                        feed.feed_type()
                    )
                }
            }
            Err(err) => println!("Unable to find feeds due to error: {}", err),
        }
    }
}
