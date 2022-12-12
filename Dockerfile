FROM rust:1-slim-buster as builder

WORKDIR /build

COPY src/ /build/src/
COPY tests/ /build/tests/
COPY bin/ /build/bin/
COPY Cargo.toml /build/Cargo.toml
COPY Cargo.lock /build/Cargo.lock
COPY build.rs /build/build.rs
COPY .version /build/.version

RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y --no-install-recommends openssh-client curl wget \
    && cd /build \
    && cargo build --release \
    && chmod a+x target/release/rymfony

FROM scratch

WORKDIR /srv

COPY --from=builder /build/target/release/rymfony /rymfony

ENTRYPOINT ["/rymfony"]
