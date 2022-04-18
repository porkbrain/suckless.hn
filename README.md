# sucklesshn.porkbrain.com
*This section is about motivation behind the project. See [tech
stack](#design).*

Some stories on [HN][hn] are frustrating and time consuming for dubious value.
I believe there are other people who would also like to see less of certain
type of content, hence *suckless hn*.

* Why doesn't this instead exist as an app into which I login as an HN user and
  it [hides][hn-hide-story] stories on my behalf?

    As a user I wouldn't login into a 3rd party app. As a developer I don't
    want to manage user credentials.

* Can I have custom filters configurable from a UI?

    Out of scope. [Create an issue][create-issue] or submit a PR if there's a
    filter you wish to use.

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
[`https://sucklesshn.porkbrain.com/+bignews`](https://sucklesshn.porkbrain.com/+bignews).
To get HN without large newspapers visit
[`https://sucklesshn.porkbrain.com/-bignews`](https://sucklesshn.porkbrain.com/-bignews).

There are also groups of filters. For example
[`https://sucklesshn.porkbrain.com/-amfg-bignews`](https://sucklesshn.porkbrain.com/-amfg-bignews)
filters out large newspapers _and_ all mentions of big tech. This also happens
to be the default view on the [homepage][homepage]. `-` modifier in a group is
conjunctive, i.e. only stories which didn't pass any of the filters are shown.
`+` modifier is disjunctive, i.e. stories which passed any of the filters are
shown. For example
[sucklesshn.porkbrain.com/`+askhn+showhn`](https://sucklesshn.porkbrain.com/+askhn+showhn)
shows "Show HN" _or_ "Ask HN" stories.

### List
**List of implemented filters:**
* [`+askhn`](https://sucklesshn.porkbrain.com/+askhn)/[`-askhn`](https://sucklesshn.porkbrain.com/-askhn)
  flags "Ask HN" titles

* [`+showhn`](https://sucklesshn.porkbrain.com/+showhn)/[`-showhn`](https://sucklesshn.porkbrain.com/-showhn)
  flags "Show HN" titles

* [`+bignews`](https://sucklesshn.porkbrain.com/+bignews)/[`-bignews`](https://sucklesshn.porkbrain.com/-bignews)
  flags urls from large news sites Bloomberg, VICE, The Guardian, WSJ, CNBC,
  BBC, Forbes, Spectator, LA Times, The Hill and NY Times. More large news may
  be added later. Any general news website which has *~60* submissions (2
  pages) in the past year falls into this category. HN search query:
  `https://hn.algolia.com/?dateRange=pastYear&page=2&prefix=true&sort=byPopularity&type=story&query=${DOMAIN}`.

* [`+amfg`](https://sucklesshn.porkbrain.com/+amfg)/[`-amfg`](https://sucklesshn.porkbrain.com/-amfg)
  flags titles which mention "Google", "Facebook", "Apple" or "Microsoft". No
  more endless Google-bashing comment binging at 3 AM. Most of the time the
  submissions are scandalous and comment sections low entropy but addictive.

* special [`+all`](https://sucklesshn.porkbrain.com/+all) front page which
  includes all HN top stories

**List of filter groups:**
* [sucklesshn.porkbrain.com/`-amfg-bignews`](https://sucklesshn.porkbrain.com/-amfg-bignews) (default)
* [sucklesshn.porkbrain.com/`+askhn+showhn`](https://sucklesshn.porkbrain.com/+askhn+showhn)

Filters in a group are alphabetically sorted ASC.

## Design
The binary is executed periodically (~ 30 min). Each generated page is an S3
object, therefore we don't need to provision a web server.

[`sqlite`][sqlite] database stores ids of top HN posts that are already
downloaded + some other data (timestamp of insertion, submission title, url,
which filters it passed).

The endpoint to query top stories on HN is
[https://hacker-news.firebaseio.com/v0/topstories.json][hn-topstories]. We
download stories which we haven't checked before. The data about a story is
available via [item endpoint][hn-item].

We check each new story against Suckless filters before inserting it into the
database table `stories`. The flags for each filter are persisted in
`story_filters` table.

Final step is generating a new html for the
[sucklesshn.porkbrain.com][suckless-hn] front pages and uploading it into an
[S3 bucket][s3-upload]. The S3 bucket is behind Cloudfront distribution to
which the `sucklesshn.porkbrain.com` DNS zone records point. We set up
different combinations of filters and upload those combinations as different S3
objects. The objects are all of `Content-type: text/html`, however they don't
have `.html` extension.

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

## Build
I run the binary on my [k8s homelab cluster][cluster] as a [cron
job](k8s/cron.yml). Originally, this ran as a cron job on my [raspberry pi
4][pi-4], which is now a node in the cluster. I still build this project for
[ARM][pi-target]. See the [`k8s`](k8s) directory for more docs about how this
project runs in the cluster.

I use a [build script](bin/build.sh) to build and test this project. First,
you'll need to install `cross`:

```bash
cargo install --git https://github.com/anupdhml/cross.git --branch master
```

We use custom [image](armv7-unknown-linux-gnueabihf/Dockerfile) for compilation
to support [OpenSSL][cross-opensll].

Next, either use the build script or directly compile for
`armv7-unknown-linux-gnueabihf`:

```bash
cross build --target armv7-unknown-linux-gnueabihf --release
```

Locally I build the [docker image with the binary](Dockerfile) and push it to
the [Docker hub][dockerhub-suckless-hn]. That's where my k8s cluster pulls it
from.

### Env
See the [`.env.example`](.env.example) file for environment variable the binary
expects.

<!-- References -->
[create-issue]: https://github.com/bausano/suckless.hn/issues/new
[cross-openssl]: https://www.reddit.com/r/rust/comments/axaq9b/opensslsys_error_when_crosscompiling_for/ehsa59c
[cross]: https://github.com/rust-embedded/cross
[hn-hide-story]: https://news.ycombinator.com/item?id=5225884
[hn-item]: https://github.com/HackerNews/API#items
[hn-topstories]: https://github.com/HackerNews/API#new-top-and-best-stories
[hn]: https://news.ycombinator.com/news
[homepage]: https://sucklesshn.porkbrain.com
[pi-4]: https://www.raspberrypi.org/products/raspberry-pi-4-model-b
[pi-target]: https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi
[s3-upload]: https://durch.github.io/rust-s3/s3/bucket/struct.Bucket.html#method.put_object_with_content_type
[sqlite]: https://github.com/rusqlite/rusqlite
[suckless-hn]: https://sucklesshn.porkbrain.com
[wayback-donate]: https://archive.org/donate
[wayback-machine-api]: https://archive.org/help/wayback_api.php
[cluster]: https://github.com/bausano/cluster
[dockerhub-suckless-hn]: https://hub.docker.com/repository/docker/porkbrain/suckless.hn
