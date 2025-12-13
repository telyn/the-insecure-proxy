FROM rust:1.75 as builder
WORKDIR /usr/src/the-insecure-proxy
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

RUN cargo fetch

COPY src/ ./src/
RUN cargo install -vv --path .


FROM debian:trixie-slim

COPY --from=builder /usr/local/cargo/bin/the-insecure-proxy /usr/local/bin/the-insecure-proxy

RUN apt-get update \
 && apt-get install -y libssl3 ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/* \
 && mkdir /the-insecure-proxy \
 && useradd the-insecure-proxy \
 && chown -R the-insecure-proxy /the-insecure-proxy

WORKDIR /the-insecure-proxy
USER the-insecure-proxy

CMD [ "/usr/local/bin/the-insecure-proxy" ]
