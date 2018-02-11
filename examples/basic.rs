extern crate feedfinder;
extern crate url;

use feedfinder::detect_feeds;
use url::Url;

fn main() {
    let url = Url::parse("https://example.com/example").expect("unable to parse url");
    let html = r#"
        <html>
            <head>
                <title>Example</title>
                <link rel="alternate" href="/posts.rss" type="application/rss+xml" />
            </head>
            <body>
                My fun page with a feed.
            </body>
        </html>"#;

    match detect_feeds(&url, html) {
        Ok(feeds) => {
            println!("Possible feeds for {}:", url);
            for feed in feeds {
                println!("* {:?}", feed);
            }
        }
        Err(err) => println!("Unable to find feeds due to error: {}", err),
    }
}
