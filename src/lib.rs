#[macro_use]
extern crate failure;
extern crate kuchiki;
extern crate url;

use url::Url;
use kuchiki::traits::*;

const MIGHT_BE_FEED: [&str; 4] = ["feed", "xml", "rss", "atom"];

#[derive(Debug, Fail, PartialEq)]
pub enum FeedFinderError {
    #[fail(display = "{}", _0)] Url(#[cause] url::ParseError),
    #[fail(display = "unable to select elements in doc")] Select,
}

#[derive(Debug, PartialEq)]
pub enum Feed {
    Rss(Url),
    Atom(Url),
    Json(Url),
    Link(Url),
}

type FeedResult = Result<Vec<Feed>, FeedFinderError>;

struct FeedFinder<'a> {
    doc: kuchiki::NodeRef,
    base_url: &'a Url,
}

pub fn detect_feeds(base_url: &Url, html: String) -> FeedResult {
    let finder = FeedFinder {
        doc: kuchiki::parse_html().one(html),
        base_url,
    };

    let sources = [FeedFinder::meta_links, FeedFinder::youtube, FeedFinder::body_links, FeedFinder::guess];
    for source in &sources {
        let candidates = source(&finder)?;
        if !candidates.is_empty() {
            return Ok(candidates);
        }
    }

    Ok(Vec::new())
}

impl<'a> FeedFinder<'a> {
    fn meta_links(&self) -> FeedResult {
        let mut feeds = vec![];
        for meta in self.doc.select("meta[rel='alternate']")
            .map_err(|_| FeedFinderError::Select)?
        {
            let attrs = meta.attributes.borrow();
            match (attrs.get("type"), attrs.get("href")) {
                (Some("application/rss+xml"), Some(href)) => {
                    feeds.push(Feed::Rss(self.base_url.join(href).map_err(FeedFinderError::Url)?))
                }
                (Some("application/atom+xml"), Some(href)) => {
                    feeds.push(Feed::Atom(self.base_url.join(href).map_err(FeedFinderError::Url)?))
                }
                (Some("application/json"), Some(href)) => {
                    feeds.push(Feed::Json(self.base_url.join(href).map_err(FeedFinderError::Url)?))
                }
                _ => (),
            }
        }

        Ok(feeds)
    }

    fn youtube(&self) -> FeedResult {
        Ok(vec![])
    }

    // Searches the body for links to things that might be feeds
    fn body_links(&self) -> FeedResult {
        let mut feeds = vec![];

        for meta in self.doc.select("a")
            .map_err(|_| FeedFinderError::Select)?
        {
            let attrs = meta.attributes.borrow();
            if let Some(ref href) = attrs.get("href") {
                if MIGHT_BE_FEED.iter().any(|hint| href.contains(hint)) {
                    feeds.push(Feed::Link(self.base_url.join(href).map_err(FeedFinderError::Url)?))
                }
            }
        }

        Ok(feeds)
    }

    fn guess(&self) -> FeedResult {
        Ok(vec![])
    }
}

#[test]
fn test_detect_meta_atom() {
    let base = Url::parse("http://example.com/").unwrap();
    let html = r#"<html><head><meta rel="alternate" type="application/atom+xml" href="http://example.com/feed.atom"></head></html>"#.to_owned();
    let url = Url::parse("http://example.com/feed.atom").unwrap();
    assert_eq!(detect_feeds(&base, html), Ok(vec![Feed::Atom(url)]));
}

#[test]
fn test_detect_meta_rss() {
    let base = Url::parse("http://example.com/").unwrap();
    let html = r#"<html><head><meta rel="alternate" type="application/rss+xml" href="http://example.com/feed.rss"></head></html>"#.to_owned();
    let url = Url::parse("http://example.com/feed.rss").unwrap();
    assert_eq!(detect_feeds(&base, html), Ok(vec![Feed::Rss(url)]));
}

#[test]
fn test_detect_meta_rss_relative() {
    let base = Url::parse("http://example.com/").unwrap();
    let html = r#"<html><head><meta rel="alternate" type="application/rss+xml" href="/feed.rss"></head></html>"#.to_owned();
    let url = Url::parse("http://example.com/feed.rss").unwrap();
    assert_eq!(detect_feeds(&base, html), Ok(vec![Feed::Rss(url)]));
}

#[test]
fn test_detect_meta_json_feed() {
    let base = Url::parse("http://example.com/").unwrap();
    let html = r#"<html><head><meta rel="alternate" type="application/json" href="http://example.com/feed.json"></head></html>"#.to_owned();
    let url = Url::parse("http://example.com/feed.json").unwrap();
    assert_eq!(detect_feeds(&base, html), Ok(vec![Feed::Json(url)]));
}

#[test]
fn test_body_link_feed() {
    let base = Url::parse("http://example.com/").unwrap();
    let html = r#"<html><body><a href="/feed.rss">RSS</a></body</html>"#.to_owned();
    let url = Url::parse("http://example.com/feed.rss").unwrap();
    assert_eq!(detect_feeds(&base, html), Ok(vec![Feed::Link(url)]));
}


