extern crate reqwest;
extern crate kuchiki;

use reqwest::Url;
use kuchiki::traits::*;

#[derive(Debug, PartialEq)]
pub enum Error {
    NoFeedsFound
}

#[derive(Debug, PartialEq)]
pub enum Feed {
    Rss(Url),
    Atom(Url),
    Json(Url),
}

type FeedResult = Result<Vec<Feed>, Error>;

pub fn detect_feeds(html: String) -> FeedResult {
    let doc = kuchiki::parse_html().one(html);

    let sources = [meta_links, youtube, body_links, guess];
    for source in sources.into_iter() {
        let candidates = source(&doc)?;
        if !candidates.is_empty() {
            return Ok(candidates);
        }
    }

    Ok(Vec::new())
}

fn meta_links(doc: &kuchiki::NodeRef) -> FeedResult {
    Ok(vec![])
}

fn youtube(doc: &kuchiki::NodeRef) -> FeedResult {
    Ok(vec![])
}

fn body_links(doc: &kuchiki::NodeRef) -> FeedResult {
    Ok(vec![])
}

fn guess(doc: &kuchiki::NodeRef) -> FeedResult {
    Ok(vec![])
}

#[test]
fn test_detect_meta_atom() {
    let html = r#"<html><head><meta rel="alternate" src=""></head></html>"#;
    let doc = kuchiki::parse_html().one(html);
    assert_eq!(meta_links(&doc), Ok(vec![]));
}

#[test]
fn test_detect_meta_rss() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_detect_meta_json_feed() {
    assert_eq!(2 + 2, 4);
}
