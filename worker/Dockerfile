# Build Stage
FROM rust:1.80.1-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /usr/
RUN USER=root cargo new app
WORKDIR /usr/app

COPY Cargo.toml Cargo.lock ./
COPY lib ./src
RUN touch -a -m ./worker/src/main.rs && cargo build --release --locked

# Bundle Stage
FROM alpine:3.20.2

ARG BUILD_NUMBER
ENV DD_VERSION="${BUILD_NUMBER}"

COPY --from=builder /usr/app/target/release/api /usr/app

CMD ["/usr/app"]