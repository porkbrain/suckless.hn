FROM balenalib/armv7hf-ubuntu

COPY target/armv7-unknown-linux-gnueabihf/release/suckless_hn /usr/local/bin/suckless_hn

RUN [ "cross-build-start" ]

RUN apt-get update

RUN [ "cross-build-end" ]

CMD suckless_hn

