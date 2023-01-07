# syntax=docker/dockerfile:1
FROM rust:latest AS builder

WORKDIR /usr/src/project

RUN set -eux; \
    apt-get install -y musl-dev youtube-dl ffmpeg libopus-dev cmake build-essential autoconf automake libtool m4; \
    cargo install cargo-chef; \
    rm -rf $CARGO_HOME/registry \

COPY . .
RUN cargo build --release

FROM ubuntu:22.04

WORKDIR /hallabong

RUN apt-get install -y youtube-dl ffmpeg libopus-dev; \
    addgroup -S hallabong; \
    adduser -S -G hallabong hallabong

COPY --from=builder /usr/src/project/target/release/hallabong ./hallabong

USER hallabong

ENV DISCORD_TOKEN YOU_MUST_SET_THIS
CMD ["./hallabong"]
