# FROM rust:1.80.1 as builder
# WORKDIR /usr/src/app
# COPY . .
# RUN cargo build --release

FROM debian:bookworm-slim
COPY target/release/vintage /usr/local/bin/
COPY node1.yml /vintage/config.yml

# CMD ["vintage", "-c", "/vintage/config.yml"]

