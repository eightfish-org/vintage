FROM rust:1.80.1 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /usr/src/app/target/release/vintage /usr/local/bin/vintage
COPY config.yml /vintage/config.yml
CMD ["vintage", "-c", "/vintage/config.yml"]

