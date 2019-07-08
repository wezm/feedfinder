# Feed Finder

A Rust crate for auto-discovering RSS, Atom, JSON feeds.

[![Build Status](https://travis-ci.org/wezm/feedfinder.svg?branch=master)](https://travis-ci.org/wezm/feedfinder)
[![Docs on docs.rs](https://docs.rs/feedfinder/badge.svg)][documentation]
[![crates.io](https://img.shields.io/crates/v/feedfinder.svg)](https://crates.io/crates/feedfinder)

## Documentation

[Documentation][documentation] is available on docs.rs.

## Features

`feedfinder` can find feeds:

* Linked via the `<link>` tag in the HTML
* Linked via `<a>` tag in the HTML
* By guessing from the software used to generate the page:
    * Tumblr
    * WordPress
    * Hugo
    * Jekyll
    * Ghost
* From YouTube:
    * channels
    * playlists
    * users

## Examples

See the [documentation] for an example as well as the [examples] directory in
the source. Examples are runnable with `cargo run --example example-name`.

## Credits

Some logic derived from [FeedFinder] in [Feedbin].

[FeedFinder]: https://github.com/feedbin/feedbin/blob/a748eb250ef1d02ecd5ee596bd5a94dac775fbd1/app/models/feed_finder.rb
[Feedbin]: https://feedbin.com/
[documentation]: https://docs.rs/crate/feedfinder/
[examples]: https://github.com/wezm/feedfinder/tree/master/examples
