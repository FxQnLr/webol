FROM debian:bookworm AS deb_extractor
RUN cd /tmp && \
    apt-get update && apt-get download \
        libc6 && \
    mkdir /dpkg && \
    for deb in *.deb; do dpkg --extract $deb /dpkg || exit 10; done

FROM lukemathwalker/cargo-chef:latest-rust-1.73.0 as chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/webol /
COPY --from=deb_extractor /dpkg /

EXPOSE 7229
ENTRYPOINT ["./webol"]