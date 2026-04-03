FROM rust:1.94.1-slim-bookworm as build
WORKDIR /src
COPY . .
RUN cargo build --release

FROM debian:bookworm-20260316-slim
RUN apt-get update  \
    && apt-get install -qqy --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /src/target/release/checker /usr/local/bin/checker

RUN useradd -m checker  \
    && chown checker:checker /usr/local/bin/checker

USER checker
ENTRYPOINT ["/usr/local/bin/checker"]