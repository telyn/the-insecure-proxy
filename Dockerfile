FROM rust:1.70 as builder
WORKDIR /usr/src/the-insecure-proxy
COPY Cargo.toml Cargo.lock .

RUN cargo fetch

COPY src/* ./src/
RUN cargo install -vv --path .


FROM debian:buster-slim

COPY --from=builder /usr/local/cargo/bin/the-insecure-proxy /usr/local/bin/the-insecure-proxy

RUN apt-get update \
 && apt-get install -y libssl1.1 ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/* \
 && mkdir /the-insecure-proxy \
 && useradd the-insecure-proxy \
 && chown -R the-insecure-proxy /the-insecure-proxy

WORKDIR /the-insecure-proxy
USER the-insecure-proxy

CMD [ "/usr/local/bin/the-insecure-proxy" ]
