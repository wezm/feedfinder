#[macro_use]
extern crate failure;
extern crate kuchiki;
extern crate reqwest;

use reqwest::Url;
use kuchiki::traits::*;

#[derive(Debug, Fail, PartialEq)]
pub enum FeedFinderError {
    #[fail(display = "{}", _0)] Url(#[cause] reqwest::UrlError),
    #[fail(display = "unable to select elements in doc")] Select,
}

#[derive(Debug, PartialEq)]
pub enum Feed {
    Rss(Url),
    Atom(Url),
    Json(Url),
}

type FeedResult = Result<Vec<Feed>, FeedFinderError>;

pub fn detect_feeds(html: String) -> FeedResult {
    let doc = kuchiki::parse_html().one(html);

    let sources = [meta_links, youtube, body_links, guess];
    for source in &sources {
        let candidates = source(&doc)?;
        if !candidates.is_empty() {
            return Ok(candidates);
        }
    }

    Ok(Vec::new())
}

fn meta_links(doc: &kuchiki::NodeRef) -> FeedResult {
    let mut feeds = vec![];
    for meta in doc.select("meta[rel='alternate']")
        .map_err(|_| FeedFinderError::Select)?
    {
        let attrs = meta.attributes.borrow();
        match (attrs.get("type"), attrs.get("href")) {
            (Some("application/rss+xml"), Some(href)) => {
                feeds.push(Feed::Rss(Url::parse(href).map_err(FeedFinderError::Url)?))
            }
            (Some("application/atom+xml"), Some(href)) => {
                feeds.push(Feed::Atom(Url::parse(href).map_err(FeedFinderError::Url)?))
            }
            (Some("application/json"), Some(href)) => {
                feeds.push(Feed::Json(Url::parse(href).map_err(FeedFinderError::Url)?))
            }
            _ => (),
        }
    }

    Ok(feeds)
}

fn youtube(_doc: &kuchiki::NodeRef) -> FeedResult {
    Ok(vec![])
}

fn body_links(_doc: &kuchiki::NodeRef) -> FeedResult {
    Ok(vec![])
}

fn guess(_doc: &kuchiki::NodeRef) -> FeedResult {
    Ok(vec![])
}

#[test]
fn test_detect_meta_atom() {
    let html = r#"<html><head><meta rel="alternate" type="application/atom+xml" href="http://example.com/feed.atom"></head></html>"#;
    let doc = kuchiki::parse_html().one(html);
    let url = Url::parse("http://example.com/feed.atom").unwrap();
    assert_eq!(meta_links(&doc), Ok(vec![Feed::Atom(url)]));
}

#[test]
fn test_detect_meta_rss() {
    let html = r#"<html><head><meta rel="alternate" type="application/rss+xml" href="http://example.com/feed.rss"></head></html>"#;
    let doc = kuchiki::parse_html().one(html);
    let url = Url::parse("http://example.com/feed.rss").unwrap();
    assert_eq!(meta_links(&doc), Ok(vec![Feed::Rss(url)]));
}

#[test]
fn test_detect_meta_json_feed() {
    let html = r#"<html><head><meta rel="alternate" type="application/json" href="http://example.com/feed.json"></head></html>"#;
    let doc = kuchiki::parse_html().one(html);
    let url = Url::parse("http://example.com/feed.json").unwrap();
    assert_eq!(meta_links(&doc), Ok(vec![Feed::Json(url)]));
}
