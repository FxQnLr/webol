FROM debian:bookworm AS deb_extractor
RUN cd /tmp && \
    apt-get update && apt-get download \
        libc6 && \
    mkdir /dpkg && \
    for deb in *.deb; do dpkg --extract $deb /dpkg || exit 10; done

FROM rust:1.80 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .


FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/webol /
COPY --from=deb_extractor /dpkg /

EXPOSE 7229
ENTRYPOINT ["./webol"]
