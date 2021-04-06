FROM balenalib/armv7hf-debian

COPY target/armv7-unknown-linux-musleabihf/release/suckless_hn /usr/local/bin/suckless_hn

RUN [ "cross-build-start" ]

RUN apt-get update
RUN ls /usr/local/bin/suckless_hn -l
RUN chmod +x /usr/local/bin/suckless_hn
RUN ls /usr/local/bin/suckless_hn -l

RUN [ "cross-build-end" ]

CMD suckless_hn

