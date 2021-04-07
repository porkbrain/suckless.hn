FROM balenalib/armv7hf-ubuntu

# take the built binary (see bin/build.sh) and copy it in the image
COPY target/armv7-unknown-linux-gnueabihf/release/suckless_hn /usr/local/bin/suckless_hn

# in this section we can run commands for ARM arch on x86 (this isn't related
# to the cross rust tool)
RUN [ "cross-build-start" ]

RUN apt-get update

RUN [ "cross-build-end" ]

CMD suckless_hn

