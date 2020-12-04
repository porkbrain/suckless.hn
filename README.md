# suckless.hn
TODO: motivation, combinations of filters

## Suckless Filters™
TODO

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

There's a helper script `deploy.sh` which compiles the binary and deploys it to the pi. It requires some env vars you can find in the `.env.deploy.example`. Rename the file to `.env.deploy` and change the values to deploy.

<!-- References -->
[pi-4]: https://www.raspberrypi.org/products/raspberry-pi-4-model-b
[pi-target]: https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi
[cross]: https://github.com/rust-embedded/cross
[sqlite]: https://github.com/rusqlite/rusqlite
[hn-topstories]: https://github.com/HackerNews/API#new-top-and-best-stories
[hn-item]: https://github.com/HackerNews/API#items
[suckless-hn]: https://suckless.hn
[wayback-machine-api]: https://archive.org/help/wayback_api.php
[wayback-donate]: https://archive.org/donate
