# suckless.hn

TODO

## Design

TODO

## Build and deploy
I run the binary periodically on my [raspberry pi 4][pi-4]. To build for the [target `armv7-unknown-linux-gnueabihf`][pi-target] we use [`cross` tool][cross].

Install `cross`.

```bash
cargo install --git https://github.com/anupdhml/cross.git --branch master
```

Compile for `armv7-unknown-linux-gnueabihf`.

```bash
cross --target armv7-unknown-linux-gnueabihf --release
```

There's a helper script `deploy.sh` which compiles the binary and deploys it to the pi. It requires some env vars you can find in the `.env.example`. Rename the file to `.env` and change the values to deploy.

<!-- References -->
[pi-4]: https://www.raspberrypi.org/products/raspberry-pi-4-model-b
[pi-target]: https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi
[cross]: https://github.com/rust-embedded/cross
