# Feed Finder

A Rust crate for auto-discovering RSS, Atom, JSON feeds.

[Documentation]

`feedfiner` can find feeds:

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

## Credits

Some logic derived from [FeedFinder] in [Feedbin]:

[FeedFinder]: https://github.com/feedbin/feedbin/blob/a748eb250ef1d02ecd5ee596bd5a94dac775fbd1/app/models/feed_finder.rb
[Feedbin]: https://feedbin.com/
[Documentation]: https://docs.rs/crate/feedfinder/
