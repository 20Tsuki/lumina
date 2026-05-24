FROM rust:1.78-slim-bookworm AS builder
WORKDIR /app
COPY apps/server/ .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/lumina-server /usr/local/bin/lumina-server
COPY apps/server/config.toml /etc/lumina/config.toml
ENV LUMINA_CONFIG=/etc/lumina/config.toml
EXPOSE 8080
VOLUME ["/data", "/media"]
CMD ["lumina-server"]
