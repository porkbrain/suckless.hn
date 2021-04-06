# suckless.hn
*This section is about motivation behind the project. See [tech stack](#design).*

Some stories on [HN][hn] are frustrating and time consuming for dubious value.
I believe there are other people who would also like to see less of certain
type of content, hence *suckless.hn*.

* Why doesn't this instead exist as an app into which I login as an HN user and
  it [hides][hn-hide-story] stories on my behalf?

    As a user I wouldn't login into a 3rd party app. As a developer I don't want to manage user credentials.

* Can I have custom filters configurable from a UI?

    Out of scope. [Create an issue][create-issue] or submit a PR if there's a filter you wish to use.

* Will you change a filter I use without my knowledge?

    I am reluctant to change the logic of a filter once it's published. However
    if it absolutely needs to happen, you'll be informed by a short update
    notice at the bottom of the page.

* Why not ML?

    I prefer a set of transparent and easily editable rules to decide what I
    don't see. Plus that's easier.

## Suckless filters
A filter is given a story data and flags the story if it passes the filter.
Feel free to [create an issue][create-issue] for any missing but useful filter.

Each filter has a two landing pages. One with only stories which were flagged,
one with anything but. This is decided by two modifiers: `+` and `-`. For
example to only see stories from large newspapers visit
[`https://suckless.hn/+bignews`](https://suckless.hn/+bignews). To get HN
without large newspapers visit
[`https://suckless.hn/-bignews`](https://suckless.hn/-bignews).

There are also groups of filters. For example
[`https://suckless.hn/-amfg-bignews`](https://suckless.hn/-amfg-bignews)
filters out large newspapers _and_ all mentions of big tech. This also happens
to be the default view on the [homepage][homepage]. `-` modifier in a group is
conjunctive, i.e. only stories which didn't pass any of the filters are shown.
`+` modifier is disjunctive, i.e. stories which passed any of the filters are
shown. For example
[suckless.hn/`+askhn+showhn`](https://suckless.hn/+askhn+showhn) shows "Show
HN" _or_ "Ask HN" stories.

### List
**List of implemented filters:**
* [`+askhn`](https://suckless.hn/+askhn)/[`-askhn`](https://suckless.hn/-askhn) flags "Ask HN" titles

* [`+showhn`](https://suckless.hn/+showhn)/[`-showhn`](https://suckless.hn/-showhn) flags "Show HN" titles

* [`+bignews`](https://suckless.hn/+bignews)/[`-bignews`](https://suckless.hn/-bignews)
  flags urls from large news sites Bloomberg, VICE, The Guardian, WSJ, CNBC,
  BBC, Forbes, Spectator, LA Times, The Hill and NY Times. More large news may
  be added later. Any general news website which has *~60* submissions (2
  pages) in the past year falls into this category. HN search query:
  `https://hn.algolia.com/?dateRange=pastYear&page=2&prefix=true&sort=byPopularity&type=story&query=${DOMAIN}`.

* [`+amfg`](https://suckless.hn/+amfg)/[`-amfg`](https://suckless.hn/-amfg)
  flags titles which mention "Google", "Facebook", "Apple" or "Microsoft". No
  more endless Google-bashing comment binging at 3 AM. Most of the time the
  submissions are scandalous and comment sections low entropy but addictive.

* special [`+all`](https://suckless.hn/+all) front page which includes all HN
  top stories

**List of filter groups:**
* [suckless.hn/`-amfg-bignews`](https://suckless.hn/-amfg-bignews) (default)
* [suckless.hn/`+askhn+showhn`](https://suckless.hn/+askhn+showhn)

Filters in a group are alphabetically sorted ASC.

## Design
The binary is executed periodically (~ 30 min). It runs on [rasbpi 4][pi-4].
The main idea is that each generated page is an S3 object, therefore we don't
need to provision a server.

[`sqlite`][sqlite] database stores ids of top HN posts that are already
downloaded + some other data (timestamp of insertion, submission title, url,
which filters it passed).

The endpoint to query top stories on HN is
[https://hacker-news.firebaseio.com/v0/topstories.json][hn-topstories]. We
download stories which we haven't checked before. The data about a story is
available via [item endpoint][hn-item].

We check each new story against Suckless Filters™ before inserting it into the
database table `stories`. The flags for each filter are persisted in
`story_filters` table.

Final step is generating a new html for the [suckless.hn][suckless-hn] front
pages and uploading it into an [S3 bucket][s3-upload]. The S3 bucket is behind
Cloudfront distribution to which the `suckless.hn` DNS zone records point. We
set up different combinations of filters and upload those combinations as
different S3 objects. The objects are all of `Content-type: text/html`, however
they don't have `.html` extension.

## Rate limiting
We handle rate limiting by simply skipping submission. Since we poll missing
stories periodically, they will be fetched eventually.

We don't need to check all top stories. We can slice the [top
stories][hn-topstories] endpoint and only download first ~ 50 entries.

[Wayback machine](#wayback-machine) has some kind of rate limiting which fails
concurrent requests. We run wayback machine `GET` requests sequentially.

## Wayback machine
We leverage [wayback machine APIs][wayback-machine-api] to provide users link
to the latest archived snapshot at the time of the submission.

Please [donate][wayback-donate] to keep Wayback machine awesome.

## Build and deploy
The binary runs periodically on [raspberry pi 4][pi-4]. To build for the target
[`armv7-unknown-linux-gnueabihf`][pi-target] we use [`cross`][cross].

Install `cross`.

```bash
cargo install --git https://github.com/anupdhml/cross.git --branch master
```

Compile for `armv7-unknown-linux-gnueabihf`.

```bash
cross build --target armv7-unknown-linux-gnueabihf --release
```

There's a helper script `deploy.sh` which compiles the binary and deploys it to
the pi. It requires env vars listed in the `.env.deploy.example`. Rename the
file to `.env.deploy` and change the values to deploy.

We use custom [image](armv7-unknown-linux-gnueabihf/Dockerfile) for compilation
to support [OpenSSL][cross-opensll].

## Cron
We setup a [`crontab`][pi-crontab] which runs the binary every 30 minutes.

```
# enters the dir where the binary is stored and runs the binary as root every
# time minute is ":00: or ":30"
# appends the logs to a file
0,30 * * * * cd /path/to/bin/dir && /usr/bin/sudo -H ./suckless_hn >>logs.txt 2>&1
```

<!-- References -->
[create-issue]: https://github.com/bausano/suckless.hn/issues/new
[cross-openssl]: https://www.reddit.com/r/rust/comments/axaq9b/opensslsys_error_when_crosscompiling_for/ehsa59c
[cross]: https://github.com/rust-embedded/cross
[hn-hide-story]: https://news.ycombinator.com/item?id=5225884
[hn-item]: https://github.com/HackerNews/API#items
[hn-topstories]: https://github.com/HackerNews/API#new-top-and-best-stories
[hn]: https://news.ycombinator.com/news
[homepage]: https://suckless.hn
[pi-4]: https://www.raspberrypi.org/products/raspberry-pi-4-model-b
[pi-crontab]: https://www.raspberrypi.org/documentation/linux/usage/cron.md
[pi-target]: https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi
[s3-upload]: https://durch.github.io/rust-s3/s3/bucket/struct.Bucket.html#method.put_object_with_content_type
[sqlite]: https://github.com/rusqlite/rusqlite
[suckless-hn]: https://suckless.hn
[wayback-donate]: https://archive.org/donate
[wayback-machine-api]: https://archive.org/help/wayback_api.php
