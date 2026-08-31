FROM rust:1.98 as builder
WORKDIR /usr/src/the-insecure-proxy
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

RUN cargo fetch

COPY src/ ./src/
RUN cargo install -vv --path .


FROM debian:trixie-slim

COPY --from=builder /usr/local/cargo/bin/the-insecure-proxy /usr/local/bin/the-insecure-proxy

# hadolint ignore=DL3008
RUN apt-get update \
 && apt-get install -y --no-install-recommends libssl3 ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/* \
 && mkdir /the-insecure-proxy \
 && useradd --uid 10001 the-insecure-proxy \
 && chown -R the-insecure-proxy /the-insecure-proxy

WORKDIR /the-insecure-proxy
USER 10001

CMD [ "/usr/local/bin/the-insecure-proxy" ]
