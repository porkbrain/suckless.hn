# Used by cross to compile openssl for rasberry pi

FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.0

RUN apt-get update -y
RUN apt-get install wget tar -y

# default dockerimage doesn't come with openssl
# https://www.reddit.com/r/rust/comments/axaq9b/opensslsys_error_when_crosscompiling_for/ehsa59c
ENV MACHINE=armv7
ENV ARCH=arm
ENV CC=arm-linux-gnueabihf-gcc
ENV OPENSSL_LIB_DIR=/tmp/openssl-1.1.1c/
ENV OPENSSL_INCLUDE_DIR=/tmp/openssl-1.1.1c/include

RUN cd /tmp && \
    wget https://www.openssl.org/source/openssl-1.1.1c.tar.gz && \
    tar xzf openssl-1.1.1c.tar.gz
RUN cd /tmp/openssl-1.1.1c && ./config shared && make

RUN openssl version
