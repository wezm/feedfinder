//! # feedfinder
//!
//! The `feedfinder` crate is for auto-discovering RSS, Atom, JSON feeds from the content of a
//! page.
//!
//! You supply the primary function [detect_feeds](fn.detect_feeds.html) with the content of
//! a HTML page and the URL of that page and it returns a list of possible feeds.
//!
//! ## About
//!
//! `feedfiner` can find feeds from these sources:
//!
//! * Linked via the `<link>` tag in the HTML
//! * Linked via `<a>` tag in the HTML
//! * By guessing from the software used to generate the page:
//!     * Tumblr
//!     * WordPress
//!     * Hugo
//!     * Jekyll
//!     * Ghost
//! * From YouTube:
//!     * channels
//!     * playlists
//!     * users
//!
//! ## Getting Started
//!
//! Add dependencies to `Cargo.toml`:
//!
//! ```toml
//! feedfinder = "0.1"
//! ```
//!
//! ## Example
//!
//! This example detects the feed linked via the `<link>` tag in the HTML.
//!
//! ```rust
//! use feedfinder::detect_feeds;
//! use url::Url;
//!
//! fn main() {
//!     let url = Url::parse("https://example.com/example").expect("unable to parse url");
//!     let html = r#"
//!         <html>
//!             <head>
//!                 <title>Example</title>
//!                 <link rel="alternate" href="/posts.rss" type="application/rss+xml" />
//!             </head>
//!             <body>
//!                 My fun page with a feed.
//!             </body>
//!         </html>"#;
//!
//!     match detect_feeds(&url, html) {
//!         Ok(feeds) => {
//!             println!("Possible feeds for {}", url);
//!             for feed in feeds {
//!                 println!("{:?}", feed);
//!             }
//!         }
//!         Err(err) => println!("Unable to find feeds due to error: {}", err),
//!     }
//! }
//! ```

#[macro_use]
extern crate failure;
use kuchiki;
use url;

use kuchiki::traits::*;
use url::Url;

const MIGHT_BE_FEED: [&str; 4] = ["feed", "xml", "rss", "atom"];

#[derive(Debug, Fail, PartialEq)]
pub enum FeedFinderError {
    #[fail(display = "{}", _0)]
    Url(#[cause] url::ParseError),
    #[fail(display = "unable to select elements in doc")]
    Select,
}

#[derive(Debug, PartialEq)]
pub enum FeedType {
    Rss,
    Atom,
    Json,
    Link,
    Guess,
}

#[derive(Debug, PartialEq)]
pub struct Feed {
    url: Url,
    type_: FeedType,
}

type FeedResult = Result<Vec<Feed>, FeedFinderError>;

struct FeedFinder<'a> {
    doc: kuchiki::NodeRef,
    base_url: &'a Url,
}

/// Find feeds in the supplied content.
///
/// The `detect_feeds` function will look for feeds:
///
/// * Linked via the `<link>` tag in the HTML
/// * Linked via `<a>` tag in the HTML
/// * By guessing from the software used to generate the page:
///     * Tumblr
///     * WordPress
///     * Hugo
///     * Jekyll
///     * Ghost
/// * From YouTube:
///     * channels
///     * playlists
///     * users
///
/// ### Parameters
///
/// `detect_feeds` takes a String of HTML content and the URL that content was retrieved
/// from. The function will not fetch the URL itself. This needs to be done by the caller
/// using a library like [reqwest](https://crates.io/crates/reqwest) or
/// [hyper](https://crates.io/crates/hyper).
///
/// ### Return value
///
/// `detect_feeds` does not access the network so the returned list of feed candidates
/// should be tested to see:
///
/// * If they actually exist.
/// * If they look like they are a feed (by checking for an XML or JSON MIME type).
///
/// The return value is wrapped in a Result, errors can occur if a candidate URL is
/// invalid or there is a problem parsing or traversing the HTML content.
///
/// ### Example
///
/// ```rust
/// use feedfinder::detect_feeds;
/// use url::Url;
///
/// fn main() {
///     let url = Url::parse("https://example.com/example").expect("unable to parse url");
///     let html = r#"
///         <html>
///             <head>
///                 <title>Example</title>
///                 <link rel="alternate" href="/posts.rss" type="application/rss+xml" />
///             </head>
///             <body>
///                 My fun page with a feed.
///             </body>
///         </html>"#;
///
///     match detect_feeds(&url, html) {
///         Ok(feeds) => {
///             println!("Possible feeds for {}", url);
///             for feed in feeds {
///                 println!("{:?}", feed);
///             }
///         }
///         Err(err) => println!("Unable to find feeds due to error: {}", err),
///     }
/// }
/// ```
pub fn detect_feeds(base_url: &Url, html: &str) -> FeedResult {
    let finder = FeedFinder {
        doc: kuchiki::parse_html().one(html),
        base_url,
    };

    let sources = [
        FeedFinder::meta_links,
        FeedFinder::youtube,
        FeedFinder::body_links,
        FeedFinder::guess,
    ];
    for source in &sources {
        let candidates = source(&finder)?;
        if !candidates.is_empty() {
            return Ok(candidates);
        }
    }

    Ok(Vec::new())
}

fn nth_path_segment(url: &Url, nth: usize) -> Option<&str> {
    if let Some(mut segments) = url.path_segments() {
        segments.nth(nth)
    } else {
        None
    }
}

impl<'a> FeedFinder<'a> {
    fn meta_links(&self) -> FeedResult {
        let mut feeds = vec![];
        for link in self
            .doc
            .select("link[rel='alternate']")
            .map_err(|_| FeedFinderError::Select)?
        {
            let attrs = link.attributes.borrow();
            match (attrs.get("type"), attrs.get("href")) {
                (Some("application/rss+xml"), Some(href)) => feeds.push(Feed {
                    url: self.base_url.join(href).map_err(FeedFinderError::Url)?,
                    type_: FeedType::Rss,
                }),
                (Some("application/atom+xml"), Some(href)) => feeds.push(Feed {
                    url: self.base_url.join(href).map_err(FeedFinderError::Url)?,
                    type_: FeedType::Atom,
                }),
                (Some("application/json"), Some(href)) => feeds.push(Feed {
                    url: self.base_url.join(href).map_err(FeedFinderError::Url)?,
                    type_: FeedType::Json,
                }),
                _ => (),
            }
        }

        Ok(feeds)
    }

    fn youtube(&self) -> FeedResult {
        let mut feeds = vec![];
        let url = self.base_url.as_str();

        if url.starts_with("https://www.youtube.com/channel/") {
            // Get the path segment after /channel/
            if let Some(id) = nth_path_segment(self.base_url, 1) {
                let feed = Url::parse(&format!(
                    "https://www.youtube.com/feeds/videos.xml?channel_id={}",
                    id
                ))
                .map_err(FeedFinderError::Url)?;
                feeds.push(Feed {
                    url: feed,
                    type_: FeedType::Atom,
                });
            }
        } else if url.starts_with("https://www.youtube.com/user/") {
            // Get the path segment after /user/
            if let Some(id) = nth_path_segment(self.base_url, 1) {
                let feed = Url::parse(&format!(
                    "https://www.youtube.com/feeds/videos.xml?user={}",
                    id
                ))
                .map_err(FeedFinderError::Url)?;
                feeds.push(Feed {
                    url: feed,
                    type_: FeedType::Atom,
                });
            }
        } else if url.starts_with("https://www.youtube.com/playlist?list=")
            || url.starts_with("https://www.youtube.com/watch")
        {
            // get the value of the list query param
            for (key, value) in self.base_url.query_pairs() {
                if key == "list" {
                    let feed = Url::parse(&format!(
                        "https://www.youtube.com/feeds/videos.xml?playlist_id={}",
                        value
                    ))
                    .map_err(FeedFinderError::Url)?;
                    feeds.push(Feed {
                        url: feed,
                        type_: FeedType::Atom,
                    });
                    break;
                }
            }
        }

        Ok(feeds)
    }

    // Searches the body for links to things that might be feeds
    fn body_links(&self) -> FeedResult {
        let mut feeds = vec![];

        for a in self.doc.select("a").map_err(|_| FeedFinderError::Select)? {
            let attrs = a.attributes.borrow();
            if let Some(href) = attrs.get("href") {
                if MIGHT_BE_FEED.iter().any(|hint| href.contains(hint)) {
                    feeds.push(Feed {
                        url: self.base_url.join(href).map_err(FeedFinderError::Url)?,
                        type_: FeedType::Link,
                    })
                }
            }
        }

        Ok(feeds)
    }

    // Well this sure isn't pretty. TODO: Clean up
    fn guess_segments(&self, feed_file: &str) -> FeedResult {
        let mut feeds = Vec::new();

        if let Some(segments) = self.base_url.path_segments() {
            let mut remaining_segments = segments.collect::<Vec<_>>();
            let mut segments = vec!["", feed_file];

            loop {
                let url = self
                    .base_url
                    .join(&segments.join("/"))
                    .map_err(FeedFinderError::Url)?;
                feeds.push(Feed {
                    url,
                    type_: FeedType::Guess,
                });

                if remaining_segments.is_empty() {
                    break;
                }

                let index = segments.len() - 1;
                let segment = remaining_segments.remove(0);
                if segment.is_empty() {
                    // Skip empty strings, which should only occur as the last element
                    break;
                }

                segments.insert(index, segment);
            }
        }

        Ok(feeds)
    }

    // Guesses the feed for some well known locations
    // Tumblr
    // Wordpress
    // Ghost
    // Jekyll
    // Hugo
    fn guess(&self) -> FeedResult {
        let markup = self.doc.to_string().to_lowercase();

        let url = if markup.contains("tumblr.com") {
            Some(self.base_url.join("/rss").map_err(FeedFinderError::Url)?)
        } else if markup.contains("wordpress") {
            Some(self.base_url.join("/feed").map_err(FeedFinderError::Url)?)
        } else if markup.contains("hugo") {
            return self.guess_segments("index.xml");
        } else if markup.contains("jekyll")
            || self
                .base_url
                .host_str()
                .map(|host| host.ends_with("github.io"))
                .unwrap_or(false)
        {
            return self.guess_segments("atom.xml");
        } else if markup.contains("ghost") {
            Some(self.base_url.join("/rss/").map_err(FeedFinderError::Url)?)
        } else {
            None
        };

        Ok(url
            .map(|url| {
                vec![Feed {
                    url,
                    type_: FeedType::Guess,
                }]
            })
            .unwrap_or_else(|| vec![]))
    }
}

impl Feed {
    // Get the URL of this feed.
    pub fn url(&self) -> &Url {
        &self.url
    }

    // Get the type of this feed.
    pub fn feed_type(&self) -> &FeedType {
        &self.type_
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_meta_atom() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><head><link rel="alternate" type="application/atom+xml" href="http://example.com/feed.atom"></head></html>"#;
        let url = Url::parse("http://example.com/feed.atom").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Atom,
            },])
        );
    }

    #[test]
    fn test_detect_meta_rss() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><head><link rel="alternate" type="application/rss+xml" href="http://example.com/feed.rss"></head></html>"#;
        let url = Url::parse("http://example.com/feed.rss").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Rss,
            },])
        );
    }

    #[test]
    fn test_detect_meta_rss_relative() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><head><link rel="alternate" type="application/rss+xml" href="/feed.rss"></head></html>"#;
        let url = Url::parse("http://example.com/feed.rss").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Rss,
            },])
        );
    }

    #[test]
    fn test_detect_meta_json_feed() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><head><link rel="alternate" type="application/json" href="http://example.com/feed.json"></head></html>"#;
        let url = Url::parse("http://example.com/feed.json").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Json,
            },])
        );
    }

    #[test]
    fn test_body_link_feed() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><body><a href="/feed/">RSS</a></body</html>"#;
        let url = Url::parse("http://example.com/feed/").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Link,
            },])
        );
    }

    #[test]
    fn test_body_link_xml() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><body><a href="/index.xml">RSS</a></body</html>"#;
        let url = Url::parse("http://example.com/index.xml").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Link,
            },])
        );
    }

    #[test]
    fn test_body_link_rss() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><body><a href="/comments.rss">RSS</a></body</html>"#;
        let url = Url::parse("http://example.com/comments.rss").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Link,
            },])
        );
    }

    #[test]
    fn test_body_link_atom() {
        let base = Url::parse("http://example.com/").unwrap();
        let html =
            r#"<html><body><a href="http://other.example.com/posts.atom">RSS</a></body</html>"#;
        let url = Url::parse("http://other.example.com/posts.atom").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Link,
            },])
        );
    }

    #[test]
    fn test_guess_tumblr() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><head><link href="http://static.tumblr.com/example/jquery.fancybox-1.3.4.css" rel="stylesheet" type="text/css"></head><body>First post!</body</html>"#;
        let url = Url::parse("http://example.com/rss").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Guess,
            },])
        );
    }

    #[test]
    fn test_guess_wordpress() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><head><meta name="generator" content="WordPress.com" /></head><body>First post!</body</html>"#;
        let url = Url::parse("http://example.com/feed").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Guess,
            },])
        );
    }

    #[test]
    fn test_guess_hugo() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><head><meta name="generator" content="Hugo 0.27.1" /></head><body>First post!</body</html>"#;
        let url = Url::parse("http://example.com/index.xml").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Guess,
            },])
        );
    }

    #[test]
    fn test_guess_jekyll() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><head></head><body><!-- Begin Jekyll SEO tag v2.3.0 -->First post!</body</html>"#;
        let url = Url::parse("http://example.com/atom.xml").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Guess,
            },])
        );
    }

    #[test]
    fn test_guess_github_io() {
        let base = Url::parse("http://example.github.io/").unwrap();
        let html = r#"<html><head></head><body>First post!</body</html>"#;
        let url = Url::parse("http://example.github.io/atom.xml").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Guess,
            },])
        );
    }

    #[test]
    fn test_guess_ghost() {
        let base = Url::parse("http://example.com/").unwrap();
        let html = r#"<html><head><meta name="generator" content="Ghost 1.21" /></head><body>First post!</body</html>"#;
        let url = Url::parse("http://example.com/rss/").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Guess,
            },])
        );
    }

    #[test]
    fn test_guess_hugo_non_root() {
        let base = Url::parse("http://example.com/blog/post/").unwrap();
        let html = r#"<html><head><meta name="generator" content="Hugo 0.27.1" /></head><body>First post!</body</html>"#;
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![
                Feed {
                    url: Url::parse("http://example.com/index.xml").unwrap(),
                    type_: FeedType::Guess,
                },
                Feed {
                    url: Url::parse("http://example.com/blog/index.xml").unwrap(),
                    type_: FeedType::Guess,
                },
                Feed {
                    url: Url::parse("http://example.com/blog/post/index.xml").unwrap(),
                    type_: FeedType::Guess,
                },
            ])
        );
    }

    #[test]
    fn test_guess_jekyll_non_root() {
        let base = Url::parse("http://example.github.io/blog/post/").unwrap();
        let html = r#"<html><head></head><body>First post!</body</html>"#;
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![
                Feed {
                    url: Url::parse("http://example.github.io/atom.xml").unwrap(),
                    type_: FeedType::Guess,
                },
                Feed {
                    url: Url::parse("http://example.github.io/blog/atom.xml").unwrap(),
                    type_: FeedType::Guess,
                },
                Feed {
                    url: Url::parse("http://example.github.io/blog/post/atom.xml").unwrap(),
                    type_: FeedType::Guess,
                },
            ])
        );
    }

    #[test]
    fn test_youtube_channel() {
        let base = Url::parse("https://www.youtube.com/channel/UCaYhcUwRBNscFNUKTjgPFiA").unwrap();
        let html = r#"<html><head></head><body>YouTube</body</html>"#;
        let url = Url::parse(
            "https://www.youtube.com/feeds/videos.xml?channel_id=UCaYhcUwRBNscFNUKTjgPFiA",
        )
        .unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Atom,
            },])
        );
    }

    #[test]
    fn test_youtube_user() {
        let base = Url::parse("https://www.youtube.com/user/wezmnet").unwrap();
        let html = r#"<html><head></head><body>YouTube</body</html>"#;
        let url = Url::parse("https://www.youtube.com/feeds/videos.xml?user=wezmnet").unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Atom,
            },])
        );
    }

    #[test]
    fn test_youtube_playlist() {
        let base =
            Url::parse("https://www.youtube.com/playlist?list=PLTOeCUgrkpMNEHx6j0vCH0cuyAIVZadnc")
                .unwrap();
        let html = r#"<html><head></head><body>YouTube</body</html>"#;
        let url = Url::parse(
            "https://www.youtube.com/feeds/videos.xml?playlist_id=PLTOeCUgrkpMNEHx6j0vCH0cuyAIVZadnc",
        ).unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Atom,
            },])
        );
    }

    #[test]
    fn test_youtube_watch_playlist() {
        let base =
            Url::parse("https://www.youtube.com/watch?v=0gjFYpvHyrY&list=FLOEg2K4TcePNx9SdGdR0zpg")
                .unwrap();
        let html = r#"<html><head></head><body>YouTube</body</html>"#;
        let url = Url::parse(
            "https://www.youtube.com/feeds/videos.xml?playlist_id=FLOEg2K4TcePNx9SdGdR0zpg",
        )
        .unwrap();
        assert_eq!(
            detect_feeds(&base, html),
            Ok(vec![Feed {
                url,
                type_: FeedType::Atom,
            },])
        );
    }
}
