# syntax=docker/dockerfile:1
FROM rust:latest AS builder

WORKDIR /usr/src/project

RUN set -eux; \
    apt-get update && apt-get install -y musl-dev youtube-dl ffmpeg libopus-dev cmake build-essential autoconf automake libtool m4;

COPY . .
RUN cargo build --release

FROM ubuntu:22.04

WORKDIR /hallabong

RUN apt-get update && apt-get install -y youtube-dl ffmpeg libopus-dev;

COPY --from=builder /usr/src/project/target/release/hallabong ./hallabong


ENV DISCORD_TOKEN YOU_MUST_SET_THIS
CMD ["./hallabong"]
