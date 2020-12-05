# suckless.hn
TODO: motivation, high level tldr

## Suckless Filters™
A filter is given story metadata and flags the story if it passes the filter. Feel free to create an issue for any missing but useful filter.

Each filter has a two landing pages. One with only stories which were flagged, one with anything but. This is decided by modifies `+` and `-`. For example to only see stories from large newspapers visit [`https://suckless.hn/+bignews`](https://suckless.hn/+bignews). To get HN without large newspapers visit [`https://suckless.hn/-bignews`](https://suckless.hn/-bignews).

There are also groups of filters. For example [`https://suckless.hn/-bignews-amgf`](https://suckless.hn/-bignews-amgf) filters out large newspapers and all mentions of big tech. This also happens to be the default on the [homepage][homepage].

**List of implemented filters:**
* `askhn` flags "Ask HN" titles
* `showhn` flags "Show HN" titles
* `bignews` flags urls from large news sites Bloomberg, VICE, The Guardian, WSJ, CNBC, BBC and Forbes.
* `amgf` flags titles which mention "Google", "Facebook", "Apple" or "Microsoft". No more endless Google-bashing comment binging at 3 AM. Too controversial.

**List of filter groups:**
* `-bignews-amgf` (default)
* `+askhn+showhn`

## Design
[`sqlite`][sqlite] database stores ids of top HN posts that are already downloaded + some other metadata (timestamp of insertion, submission title, url, which filters it passed).

The endpoint to query top stories on HN is [https://hacker-news.firebaseio.com/v0/topstories.json][hn-topstories]. Periodically we check each story in this index which we haven't checked before or that has passed [Suckless Filters™](#suckless-filters). The data about a story is available via [item endpoint][hn-item].

We check each new story against Suckless Filters™ before inserting it into the database.

Final step is generating a new html for the [suckless.hn][suckless-hn] front page and uploading it into S3 bucket. The S3 bucket is behind Cloudfront distribution to which the `suckless.hn` zone records point. We set up different combinations of filters and upload those combinations as different S3 objects.

## Rate limiting
We handle rate limiting by simply skipping submission. When we start getting 429 errors, the binary terminates and we expect `cron` to again run it at a later point.

Also we don't need to check all top stories. We can slice the [top stories][hn-topstories] endpoint and only consider first ~ 30 entries.

## Wayback machine
We leverage [wayback machine APIs][wayback-machine-api] to provide users link to the latest archived snapshot at the time of the submission.

Please [donate][wayback-donate] to keep Wayback machine awesome.

## Build and deploy
I run the binary periodically on my [raspberry pi 4][pi-4]. To build for the target [`armv7-unknown-linux-gnueabihf`][pi-target] we use [`cross`][cross].

Install `cross`.

```bash
cargo install --git https://github.com/anupdhml/cross.git --branch master
```

Compile for `armv7-unknown-linux-gnueabihf`.

```bash
cross build --target armv7-unknown-linux-gnueabihf --release
```

There's a helper script `deploy.sh` which compiles the binary and deploys it to the pi. It requires env vars listed in the `.env.deploy.example`. Rename the file to `.env.deploy` and change the values to deploy.

<!-- References -->
[homepage]: https://suckless.hn
[pi-4]: https://www.raspberrypi.org/products/raspberry-pi-4-model-b
[pi-target]: https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi
[cross]: https://github.com/rust-embedded/cross
[sqlite]: https://github.com/rusqlite/rusqlite
[hn-topstories]: https://github.com/HackerNews/API#new-top-and-best-stories
[hn-item]: https://github.com/HackerNews/API#items
[suckless-hn]: https://suckless.hn
[wayback-machine-api]: https://archive.org/help/wayback_api.php
[wayback-donate]: https://archive.org/donate
