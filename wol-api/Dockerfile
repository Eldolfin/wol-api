FROM rust:1.67 AS builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
WORKDIR /app
RUN apt-get update \
  && apt-get install -y curl \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/wol-relay-server /usr/local/bin/wol-relay-server
CMD ["wol-relay-server", "-c", "/app/config.yaml"]

