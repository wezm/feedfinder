extern crate feedfinder;
extern crate url;

use url::Url;
use std::io::Read;

// This example can be used to list the feeds found at a URL by combining it with
// curl. For example:
//
// URL=http://example.com/ ; curl "$URL" | cargo run --example cli "$URL"
fn main() {
    for arg in std::env::args().skip(1).take(1) {
        let url = Url::parse(&arg).expect("unable to parse URL");

        // Read html from stdin
        let mut html = String::new();
        std::io::stdin().read_to_string(&mut html).expect("error reading HTML from stdin");

        match feedfinder::detect_feeds(&url, &html) {
            Ok(feeds) => {
                println!("Possible feeds for {}", url);
                for feed in feeds {
                    println!("{:?}", feed);
                }
            }
            Err(err) => println!("Unable to find feeds due to error: {}", err),
        }
    }
}
